use jiff::{Zoned, tz};
use polars::prelude::*;

const DIVIDEND_TRACING_YEARS: u8 = 10;

fn get_year_expr() -> Expr {
    let now = Zoned::now()
        .with_time_zone(tz::offset(8).to_time_zone())
        .date();
    lit(now.year() - 1)
}

pub fn filter(lf: LazyFrame) -> LazyFrame {
    let lf = lf.with_column(
        col("CashDividendPaymentDate")
            .str()
            .to_date(StrptimeOptions {
                strict: false,
                ..Default::default()
            })
            .dt()
            .year()
            .alias("payment_year"),
    );

    let expected_year = lf
        .clone()
        .group_by([col("stock_id")])
        .agg([col("payment_year").max().alias("max_year")])
        .with_column(get_year_expr().alias("expected_year"))
        .with_column(
            when(col("max_year").gt(col("expected_year")))
                .then(col("max_year"))
                .otherwise(col("expected_year"))
                .alias("end_year"),
        )
        .with_column((col("end_year") - lit(DIVIDEND_TRACING_YEARS - 1)).alias("start_year"))
        .select([col("stock_id"), col("end_year"), col("start_year")]);

    lf.join(
        expected_year,
        [col("stock_id")],
        [col("stock_id")],
        JoinArgs::new(JoinType::Left),
    )
    .filter(
        col("payment_year")
            .gt_eq(col("start_year"))
            .and(col("payment_year").lt_eq(col("end_year"))),
    )
    .group_by([col("stock_id")])
    .agg([col("payment_year").n_unique().alias("year_count")])
    .filter(col("year_count").gt_eq(lit(DIVIDEND_TRACING_YEARS)))
    .select([col("stock_id")])
}
