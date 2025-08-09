use georm::{Defaultable, Georm};
use sqlx::types::BigDecimal;

mod models;
use models::{Product, ProductDefault};

#[sqlx::test()]
async fn create_without_generated_values(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let base = ProductDefault {
        name: "Desktop".to_owned(),
        price: BigDecimal::from(2000),
        discount_percent: 5,
        sku_number: None,
    };
    let result = base.create(&pool).await?;
    assert_eq!(BigDecimal::from(1900), result.final_price.unwrap());
    Ok(())
}

#[sqlx::test()]
async fn create_with_manual_generated_value(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let base = ProductDefault {
        name: "Monitor".to_owned(),
        price: BigDecimal::from(750),
        discount_percent: 10,
        sku_number: Some(12345),
    };
    let result = base.create(&pool).await?;
    assert_eq!(12345, result.sku_number);
    assert_eq!("Monitor", result.name);
    Ok(())
}

#[sqlx::test(fixtures("generated"))]
async fn update_does_not_change_generated_always_field(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let products = Product::find_all(&pool).await?;
    dbg!(&products);
    let mut product = products.first().unwrap().clone();
    let original_final_price = product.clone().final_price;
    product.name = "Gaming Laptop".to_owned();
    product.final_price = Some(BigDecimal::from(1000000));
    dbg!(&product);
    let updated = product.update(&pool).await?;
    assert_eq!(original_final_price, updated.final_price);
    assert_eq!("Gaming Laptop", updated.name);
    Ok(())
}

#[sqlx::test(fixtures("generated"))]
async fn upsert_handles_generated_fields(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let product = Product::find_by_name("Laptop".to_owned(), &pool).await?;
    let mut modified_product = product.clone();
    modified_product.price = BigDecimal::from(1200);

    let upserted = modified_product.upsert(&pool).await?;

    // price is updated
    assert_eq!(upserted.price, BigDecimal::from(1200));
    // final_price is re-calculated by the DB
    // 1200 * (1 - 10 / 100.0) = 1080
    assert_eq!(BigDecimal::from(1080), upserted.final_price.unwrap());

    Ok(())
}
