mod balance_sheet;
mod dividend;
mod financial_statements;

use polars::prelude::*;

pub trait DefensiveInvesting {
    fn filter_balance_sheet(self) -> PolarsResult<LazyFrame>;
    fn filter_dividend(self) -> PolarsResult<LazyFrame>;
    fn filter_financial_statements(self) -> PolarsResult<LazyFrame>;
}

impl DefensiveInvesting for LazyFrame {
    fn filter_balance_sheet(self) -> PolarsResult<LazyFrame> {
        balance_sheet::filter(self)
    }

    fn filter_dividend(self) -> PolarsResult<LazyFrame> {
        dividend::filter(self)
    }

    fn filter_financial_statements(self) -> PolarsResult<LazyFrame> {
        financial_statements::filter(self)
    }
}
