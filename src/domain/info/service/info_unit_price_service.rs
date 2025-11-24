use anyhow::Result;
use serde_json::Value;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_api_repository_trait::InfoUnitPriceApiRepository;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::core::persistence::info::fixed::unit_price::info_unit_price_repository::InfoUnitPriceRepository;
use crate::domain::info::dto::info_unit_price_upsert_request::InfoUnitPriceUpsertRequest;
use validator::Validate;

pub async fn get_info_unit_prices() -> Result<InfoUnitPriceEntity> {
    let repo = InfoUnitPriceRepository::new();
    get_info_unit_prices_with_repo(&repo).await
}

pub async fn upsert_info_unit_prices(req: InfoUnitPriceUpsertRequest) -> Result<Value> {
    req.validate()?;
    let repo = InfoUnitPriceRepository::new();
    upsert_info_unit_prices_with_repo(&repo, req).await
}

async fn get_info_unit_prices_with_repo<R: InfoUnitPriceApiRepository>(
    repo: &R,
) -> Result<InfoUnitPriceEntity> {
    let entity = repo.read()?;
    Ok(entity)
}

async fn upsert_info_unit_prices_with_repo<R: InfoUnitPriceApiRepository>(
    repo: &R,
    req: InfoUnitPriceUpsertRequest,
) -> Result<Value> {
    let mut unit_prices = repo.read()?;
    unit_prices.apply_update(req);

    repo.update(&unit_prices)?;

    Ok(serde_json::json!({
        "message": "Unit prices updated successfully",
        "updated_at": unit_prices.updated_at.to_rfc3339(),
    }))
}
