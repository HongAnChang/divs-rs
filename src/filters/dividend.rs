use jiff::{Zoned, tz};
use polars::prelude::*;

const DIVIDEND_TRACING_YEARS: u8 = 10;

fn get_year_expr() -> Expr {
    let now = Zoned::now()
        .with_time_zone(tz::offset(8).to_time_zone())
        .date();
    lit(now.year() - 1)
}

pub fn filter(lf: LazyFrame) -> PolarsResult<LazyFrame> {
    let lf = lf
        .with_columns(vec![
            col("CashDividendPaymentDate")
                .str()
                .to_date(StrptimeOptions {
                    strict: false,
                    ..Default::default()
                })
                .dt()
                .year()
                .alias("payment_year"),
        ])
        .with_columns(vec![
            col("payment_year")
                .max()
                .over([col("stock_id")])?
                .alias("max_year"),
            get_year_expr().alias("expected_year"),
        ])
        .with_columns(vec![
            when(col("max_year").gt(col("expected_year")))
                .then(col("max_year"))
                .otherwise(col("expected_year"))
                .alias("end_year"),
        ])
        .with_columns(vec![
            (col("end_year") - lit(DIVIDEND_TRACING_YEARS - 1)).alias("start_year"),
        ])
        .filter(col("payment_year").gt_eq(col("start_year")))
        .filter(col("payment_year").lt_eq(col("end_year")))
        .group_by([col("stock_id")])
        .agg([col("payment_year").n_unique().alias("year_count")])
        .filter(col("year_count").eq(lit(DIVIDEND_TRACING_YEARS)))
        .select([col("stock_id")]);

    Ok(lf)
}
