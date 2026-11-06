use crate::sources::twse;
use polars::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize)]
struct EmbedField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Serialize)]
struct Embed {
    title: String,
    color: u32,
    fields: Vec<EmbedField>,
}

#[derive(Serialize)]
struct WebhookPayload {
    embeds: Vec<Embed>,
}

pub fn check() -> Result<(), Box<dyn std::error::Error>> {
    let safe_price_lf = LazyFrame::scan_parquet("safe_price.parquet".into(), Default::default())?
        .with_columns(vec![lit(false).alias("notified")]);

    let path = "daily_record.parquet";
    let has_record = Path::new(path).is_file();

    let mut daily_record_lf = if has_record {
        LazyFrame::scan_parquet(path.into(), Default::default())?
    } else {
        safe_price_lf.clone()
    };

    if has_record {
        let diff_df = daily_record_lf
            .clone()
            .join(
                safe_price_lf.clone(),
                [col("stock_id"), col("SafePrice")],
                [col("stock_id"), col("SafePrice")],
                JoinArgs::new(JoinType::Anti),
            )
            .collect()?;

        if diff_df.height() != 0 {
            daily_record_lf = safe_price_lf;
        }
    }

    unsafe {
        std::env::set_var("POLARS_FMT_MAX_ROWS", "-1");
        std::env::set_var("POLARS_FMT_MAX_COLS", "-1");
    }

    let mut daily_record_df = daily_record_lf.collect()?;
    println!("{}", daily_record_df);

    let daily_price_lf = twse::fetch_daily_price()?;

    daily_record_df = daily_record_df
        .lazy()
        .join(
            daily_price_lf,
            [col("stock_id")],
            [col("stock_id")],
            JoinArgs::new(JoinType::Inner),
        )
        .with_columns(vec![
            (col("Price") / col("SafePrice") - lit(1.0)).alias("PremiumRate"),
        ])
        .collect()?;

    // 過濾出符合條件的訊息
    let message_df = daily_record_df
        .clone()
        .lazy()
        .filter(col("notified").eq(false))
        .filter(col("PremiumRate").lt_eq(0.0))
        .drop(cols(["notified"]))
        .sort(
            ["PremiumRate"],
            SortMultipleOptions::new().with_order_descending(false),
        )
        .collect()?;

    if message_df.height() != 0 {
        discord_notify(&message_df)?;
    }

    let mut final_df = daily_record_df
        .lazy()
        .with_columns(vec![col("PremiumRate").lt_eq(0.0).alias("notified")])
        .drop(cols(["Price", "PremiumRate"]))
        .collect()?;

    let mut file = std::fs::File::create(path)?;
    ParquetWriter::new(&mut file).finish(&mut final_df)?;

    Ok(())
}

fn discord_notify(message_df: &DataFrame) -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let url = match std::env::var("DISCORD_WEBHOOK_URL") {
        Ok(val) => val,
        Err(_) => {
            println!("未設定 Discord Webhook 網址");
            return Ok(());
        }
    };

    let mut industry_category_dict = HashMap::new();
    let categories_data = [
        ("01", "水泥工業"),
        ("02", "食品工業"),
        ("03", "塑膠工業"),
        ("04", "紡織纖維"),
        ("05", "電機機械"),
        ("06", "電器電纜"),
        ("07", "化學生技醫療"),
        ("08", "玻璃陶瓷"),
        ("09", "造紙工業"),
        ("10", "鋼鐵工業"),
        ("11", "橡膠工業"),
        ("12", "汽車工業"),
        ("13", "電子工業"),
        ("14", "建材營造"),
        ("15", "航運業"),
        ("16", "觀光餐旅"),
        ("17", "金融保險"),
        ("18", "貿易百貨"),
        ("19", "綜合"),
        ("20", "其他"),
        ("21", "化學工業"),
        ("22", "生技醫療業"),
        ("23", "油電燃氣業"),
        ("24", "半導體業"),
        ("25", "電腦及週邊設備業"),
        ("26", "光電業"),
        ("27", "通信網路業"),
        ("28", "電子零組件業"),
        ("29", "電子通路業"),
        ("30", "資訊服務業"),
        ("31", "其他電子業"),
        ("32", "文化創意業"),
        ("33", "農業科技業"),
        ("34", "電子商務"),
        ("35", "綠能環保"),
        ("36", "數位雲端"),
        ("37", "運動休閒"),
        ("38", "居家生活"),
    ];
    for (k, v) in categories_data {
        industry_category_dict.insert(k, v);
    }

    let stock_ids = message_df.column("stock_id")?.str()?;
    let names = message_df.column("name")?.str()?;
    let categories = message_df.column("industry")?.str()?;
    let prices = message_df.column("Price")?.f64()?;
    let safe_prices = message_df.column("SafePrice")?.f64()?;
    let premium_rates = message_df.column("PremiumRate")?.f64()?;

    let mut fields = Vec::new();

    for i in 0..message_df.height() {
        let stock_id = stock_ids.get(i).unwrap_or("");
        let name = names.get(i).unwrap_or("");
        let category = categories.get(i).unwrap_or("");
        let price = prices.get(i).unwrap_or(0.0);
        let safe_price = safe_prices.get(i).unwrap_or(0.0);
        let premium_rate = premium_rates.get(i).unwrap_or(0.0);

        let cat_name = industry_category_dict.get(category).unwrap_or(&"未分類");

        fields.push(EmbedField {
            name: format!("📈 {} {}（{}）", stock_id, name, cat_name),
            value: format!(
                "**${:.2}** / ${:.2} ({:.2}%)",
                price, safe_price, -premium_rate * 100f64
            ),
            inline: true,
        });
    }

    let payload = WebhookPayload {
        embeds: vec![Embed {
            title: "防禦型選股報告".to_string(),
            color: 0x28A745,
            fields,
        }],
    };

    let response = ureq::post(&url).send_json(&payload);

    match response {
        Ok(_) => println!("✅ 訊息已成功傳送！"),
        Err(err) => println!("❌ {}", err)
    }

    Ok(())
}
