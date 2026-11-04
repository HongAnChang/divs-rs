use crate::filters::DefensiveInvesting;
use polars::prelude::*;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    //     let info_lf = LazyFrame::scan_parquet("stock_info.parquet".into(), Default::default())?;
    //
    //     let bs_lf =
    //         LazyFrame::scan_parquet("TaiwanStockBalanceSheet.parquet".into(), Default::default())?
    //             .filter_balance_sheet();
    //
    //     let div_lf = LazyFrame::scan_parquet("TaiwanStockDividend.parquet".into(), Default::default())?
    //         .filter_dividend();

    let fs_lf = LazyFrame::scan_parquet(
        "TaiwanStockFinancialStatements.parquet".into(),
        Default::default(),
    )?
    .filter_financial_statements();

    println!("{:?}", fs_lf.collect()?);

    Ok(())
}
