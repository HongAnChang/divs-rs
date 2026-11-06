use jiff::{Zoned, civil, tz};
use polars::prelude::*;

const TRACING_YEARS: u8 = 10;
const MIN_REVENUE: i64 = 20_000_000_000;
const MIN_GROWTH_RATE: f64 = 1.0 / 3.0;

fn get_date_expr() -> Expr {
    let now = Zoned::now()
        .with_time_zone(tz::offset(8).to_time_zone())
        .date();

    let current_year = now.year();

    let (y, m, d) = if now < civil::date(current_year, 4, 1) {
        (current_year - 1, 9, 30)
    } else if now < civil::date(current_year, 5, 16) {
        (current_year - 1, 12, 31)
    } else if now < civil::date(current_year, 8, 15) {
        (current_year, 3, 31)
    } else if now < civil::date(current_year, 11, 15) {
        (current_year, 6, 30)
    } else {
        (current_year, 9, 30)
    };

    let args = DatetimeArgs::new(lit(y), lit(m), lit(d));
    datetime(args).cast(DataType::Date)
}

pub fn filter(lf: LazyFrame) -> PolarsResult<LazyFrame> {
    let lf = lf
        .rename(
            ["EquityAttributableToOwnersOfParent"],
            ["ParentCompanyTotalConsolidatedProfit"],
            false,
        )
        .with_columns(vec![col("date").str().to_date(Default::default())])
        .with_columns(vec![
            col("date").max().over([col("stock_id")])?.alias("max_date"),
            get_date_expr().alias("expected_date"),
        ])
        .with_columns(vec![
            when(col("max_date").gt(col("expected_date")))
                .then(col("max_date"))
                .otherwise(col("expected_date"))
                .alias("end_date"),
        ])
        .with_columns(vec![
            col("end_date")
                .dt()
                .offset_by(lit(format!("-{}q", TRACING_YEARS * 4 - 1)))
                .alias("start_date"),
        ])
        .filter(col("date").gt_eq(col("start_date")))
        .filter(col("date").lt_eq(col("end_date")))
        .with_columns(vec![
            col("date")
                .n_unique()
                .over([col("stock_id")])?
                .alias("date_count"),
        ])
        .filter(col("date_count").eq(lit(TRACING_YEARS * 4)))
        .sort(
            ["stock_id", "date"],
            SortMultipleOptions::new().with_order_descending_multi([false, true]),
        )
        .filter(
            col("Revenue")
                .head(Some(4))
                .sum()
                .over([col("stock_id")])?
                .gt_eq(MIN_REVENUE),
        )
        .with_columns(vec![
            ((col("IncomeAfterTaxes")
                .cum_count(false)
                .over([col("stock_id")])?
                - lit(1))
                / lit(4))
            .alias("year_group"),
        ])
        .with_columns(vec![
            col("IncomeAfterTaxes")
                .sum()
                .over([col("stock_id"), col("year_group")])?
                .gt(0.0)
                .alias("is_year_profitable"),
        ])
        .filter(
            col("is_year_profitable")
                .all(true)
                .over([col("stock_id")])?,
        )
        .with_columns(vec![
            when(col("EPS").eq(0.0))
                .then(lit(NULL))
                .otherwise(col("IncomeAfterTaxes") / col("EPS"))
                .slice(0, 1)
                .first()
                .over([col("stock_id")])?
                .alias("avg_recent_shares"),
            when(col("EPS").eq(0.0))
                .then(lit(NULL))
                .otherwise(col("IncomeAfterTaxes") / col("EPS"))
                .slice(-12, 1)
                .first()
                .over([col("stock_id")])?
                .alias("avg_old_shares"),
            (col("ParentCompanyTotalConsolidatedProfit")
                .head(Some(12))
                .sum()
                / lit(3.0))
            .over([col("stock_id")])?
            .alias("avg_recent_3yrs_profit"),
            (col("ParentCompanyTotalConsolidatedProfit")
                .tail(Some(12))
                .sum()
                / lit(3.0))
            .over([col("stock_id")])?
            .alias("avg_old_3yrs_profit"),
        ])
        .with_columns(vec![
            (col("avg_recent_3yrs_profit") / col("avg_recent_shares")).alias("avg_recent_3yrs_eps"),
            (col("avg_old_3yrs_profit") / col("avg_old_shares")).alias("avg_old_3yrs_eps"),
        ])
        .filter(col("avg_recent_3yrs_eps").gt(0.0))
        .filter(col("avg_old_3yrs_eps").gt(0.0))
        .filter(
            col("avg_recent_3yrs_eps").gt_eq(col("avg_old_3yrs_eps") * lit(1.0 + MIN_GROWTH_RATE)),
        )
        .select([col("stock_id"), col("avg_recent_3yrs_profit")])
        .unique(None, UniqueKeepStrategy::Any);

    Ok(lf)
}
