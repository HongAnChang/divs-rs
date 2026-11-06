use jiff::{Zoned, civil, tz};
use polars::prelude::*;

const MIN_CURRENT_RATIO: f64 = 2.0;

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
    let expected_date = get_date_expr();

    let lf = lf
        .with_columns(vec![col("date").str().to_date(Default::default())])
        .filter(col("date").eq(col("date").max().over([col("stock_id")])?))
        .filter(col("date").gt_eq(expected_date))
        .filter(col("CurrentAssets").gt_eq(col("CurrentLiabilities") * lit(MIN_CURRENT_RATIO)))
        .filter(
            (col("CurrentAssets") - col("CurrentLiabilities")).gt_eq(col("NoncurrentLiabilities")),
        )
        .select([
            col("stock_id"),
            col("EquityAttributableToOwnersOfParent"),
            col("CapitalStock"),
            col("OrdinaryShare"),
            col("NumberOfSharesInEntityHeldByEntityAndByItsSubsidiaries"),
        ]);

    Ok(lf)
}
