use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, ValueEnum)]
pub enum AllowedDataset {
    TaiwanStockBalanceSheet,
    TaiwanStockDividend,
    TaiwanStockFinancialStatements,
}

impl AllowedDataset {
    pub fn as_str(&self) -> &str {
        match self {
            AllowedDataset::TaiwanStockBalanceSheet => "TaiwanStockBalanceSheet",
            AllowedDataset::TaiwanStockDividend => "TaiwanStockDividend",
            AllowedDataset::TaiwanStockFinancialStatements => "TaiwanStockFinancialStatements",
        }
    }
}

#[derive(Parser)]
pub struct Args {
    #[arg(value_enum)]
    pub dataset: AllowedDataset,
}
