use jiff::{Zoned, civil, tz};
use polars::prelude::*;

const MIN_CURRENT_RATIO: f64 = 2.0;

fn get_date_expr() -> Expr {
    let now = Zoned::now()
        .with_time_zone(tz::offset(8).to_time_zone())
        .date();

    if now < civil::date(now.year(), 4, 1) {
        return lit(format!("{}-09-30", now.year() - 1));
    } else if now < civil::date(now.year(), 5, 16) {
        return lit(format!("{}-12-31", now.year() - 1));
    } else if now < civil::date(now.year(), 8, 15) {
        return lit(format!("{}-03-31", now.year()));
    } else if now < civil::date(now.year(), 11, 15) {
        return lit(format!("{}-06-30", now.year()));
    }

    lit(format!("{}-09-30", now.year()))
}

pub fn filter(lf: LazyFrame) -> LazyFrame {
    lf.sort(
        ["stock_id", "date"],
        SortMultipleOptions::new().with_order_descending_multi([false, true]),
    )
    .unique(Some(cols(["stock_id"])), UniqueKeepStrategy::First)
    .filter(
        col("date")
            .gt_eq(get_date_expr())
            .and(
                col("CurrentAssets")
                    .fill_null(0)
                    .gt_eq(col("CurrentLiabilities").fill_null(0) * lit(MIN_CURRENT_RATIO)),
            )
            .and(
                (col("CurrentAssets").fill_null(0) - col("CurrentLiabilities").fill_null(0))
                    .gt_eq(col("NoncurrentLiabilities").fill_null(0)),
            ),
    )
    .select([
        col("stock_id"),
        col("EquityAttributableToOwnersOfParent"),
        col("CapitalStock"),
        col("OrdinaryShare"),
        col("NumberOfSharesInEntityHeldByEntityAndByItsSubsidiaries"),
    ])
}
