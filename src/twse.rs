use polars::prelude::*;
use std::{fs::File, io::Cursor};

pub fn fetch_company_info() -> Result<(), Box<dyn std::error::Error>> {
    let uri = "https://openapi.twse.com.tw/v1/opendata/t187ap03_L";

    /*
     * NOTE:
     * 實際使用中若有遇到網路問題，會增加重試機制。
     * 雖然很簡單，但我很懶，目前不做。
     */
    let mut response = ureq::get(uri).call()?;
    let inner = response.body_mut().read_to_vec()?;

    let reader = Cursor::new(inner);
    let df = JsonReader::new(reader).finish()?;

    let mut df = df
        .lazy()
        .select([
            col("公司代號").alias("stock_id"),
            col("公司簡稱").alias("name"),
            col("產業別").alias("industry"),
            col("已發行普通股數或TDR原股發行股數")
                .cast(DataType::Float64)
                .alias("issued_shares"),
        ])
        .collect()?;

    let writer = File::create("stock_info.parquet")?;
    ParquetWriter::new(writer).finish(&mut df)?;
    Ok(())
}
