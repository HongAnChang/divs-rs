use polars::prelude::*;

mod balance_sheet;
mod dividend;
mod financial_statements;

pub trait DefensiveInvesting {
    fn filter_balance_sheet(self) -> Self;
    fn filter_dividend(self) -> Self;
    fn filter_financial_statements(self) -> Self;
}

impl DefensiveInvesting for LazyFrame {
    fn filter_balance_sheet(self) -> Self {
        balance_sheet::filter(self)
    }

    fn filter_dividend(self) -> Self {
        dividend::filter(self)
    }

    fn filter_financial_statements(self) -> Self {
        financial_statements::filter(self)
    }
}
