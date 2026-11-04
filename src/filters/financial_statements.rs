use jiff::{Zoned, civil, tz};
use polars::prelude::*;

const TRACING_YEARS: u8 = 10;

fn get_date_expr() -> Expr {
    let now = Zoned::now()
        .with_time_zone(tz::offset(8).to_time_zone())
        .date();

    let date = if now < civil::date(now.year(), 4, 1) {
        lit(format!("{}-09-30", now.year() - 1))
    } else if now < civil::date(now.year(), 5, 16) {
        lit(format!("{}-12-31", now.year() - 1))
    } else if now < civil::date(now.year(), 8, 15) {
        lit(format!("{}-03-31", now.year()))
    } else if now < civil::date(now.year(), 11, 15) {
        lit(format!("{}-06-30", now.year()))
    } else {
        lit(format!("{}-09-30", now.year()))
    };

    date.str().to_date(Default::default())
}

pub fn filter(lf: LazyFrame) -> LazyFrame {
    let lf = lf
        .rename(
            ["EquityAttributableToOwnersOfParent"],
            ["ParentCompanyTotalConsolidatedProfit"],
            false
        )
        .with_column(col("date").str().to_date(Default::default()));

    let expected_date = lf
        .clone()
        .group_by([col("stock_id")])
        .agg([col("date").max().alias("max_date")])
        .with_column(get_date_expr().alias("expected_date"))
        .with_column(
            when(col("max_date").gt(col("expected_date")))
                .then(col("max_date"))
                .otherwise(col("expected_date"))
                .alias("end_date"),
        )
        .with_column(
            col("end_date")
                .dt()
                .offset_by(lit(format!("-{}q", TRACING_YEARS * 4 - 1)))
                .alias("start_date"),
        )
        .select([col("stock_id"), col("end_date"), col("start_date")]);

    let complete_lf = lf
        .clone()
        .join(
            expected_date,
            [col("stock_id")],
            [col("stock_id")],
            JoinArgs::new(JoinType::Left),
        )
        .filter(
            col("date")
                .gt_eq(col("start_date"))
                .and(col("date").lt_eq(col("end_date"))),
        )
        .group_by([col("stock_id")])
        .agg([
            col("date").n_unique().alias("date_count"),
            col("start_date").first().alias("start_date"),
            col("end_date").first().alias("end_date"),
        ])
        .filter(col("date_count").gt_eq(lit(TRACING_YEARS * 4)))
        .select([col("stock_id"), col("start_date"), col("end_date")]);

    lf.join(
        complete_lf,
        [col("stock_id")],
        [col("stock_id")],
        JoinArgs::new(JoinType::Inner),
    )
    .filter(
        col("date")
            .gt_eq(col("start_date"))
            .and(col("date").lt_eq(col("end_date"))),
    )
}
