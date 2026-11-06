use crate::filters::DefensiveInvesting;
use polars::prelude::*;

pub fn calculate() -> Result<(), Box<dyn std::error::Error>> {
    let info_lf = LazyFrame::scan_parquet("stock_info.parquet".into(), Default::default())?;

    let bs_lf =
        LazyFrame::scan_parquet("TaiwanStockBalanceSheet.parquet".into(), Default::default())?
            .filter_balance_sheet()?;

    let div_lf = LazyFrame::scan_parquet("TaiwanStockDividend.parquet".into(), Default::default())?
        .filter_dividend()?;

    let fs_lf = LazyFrame::scan_parquet(
        "TaiwanStockFinancialStatements.parquet".into(),
        Default::default(),
    )?
    .filter_financial_statements()?;

    let res_lf = info_lf
        .join(
            bs_lf,
            [col("stock_id")],
            [col("stock_id")],
            JoinArgs::new(JoinType::Inner),
        )
        .join(
            div_lf,
            [col("stock_id")],
            [col("stock_id")],
            JoinArgs::new(JoinType::Inner),
        )
        .join(
            fs_lf,
            [col("stock_id")],
            [col("stock_id")],
            JoinArgs::new(JoinType::Inner),
        )
        .with_columns(vec![
            (col("issued_shares") - col("NumberOfSharesInEntityHeldByEntityAndByItsSubsidiaries"))
                .alias("SharesOutstanding"),
        ])
        .with_columns(vec![
            (col("avg_recent_3yrs_profit") / col("SharesOutstanding")).alias("EPS"),
            ((col("EquityAttributableToOwnersOfParent")
                - (col("CapitalStock") - col("OrdinaryShare")))
                / col("SharesOutstanding"))
            .alias("BVPS"),
        ])
        .with_columns(vec![
            when((col("EPS").gt(0.0)).and(col("BVPS").gt(0.0)))
                .then((lit(27.5) * col("EPS") * col("BVPS")).sqrt())
                .otherwise(lit(NULL))
                .alias("SafePrice"),
        ])
        .select([
            col("stock_id"),
            col("name"),
            col("industry"),
            col("SafePrice"),
        ]);

    unsafe {
        std::env::set_var("POLARS_FMT_MAX_ROWS", "-1");
        std::env::set_var("POLARS_FMT_MAX_COLS", "-1");
    }

    println!(
        "{:?}",
        res_lf.collect()?.sort(
            ["stock_id"],
            SortMultipleOptions::new().with_order_descending(false)
        )
    );

    Ok(())
}
