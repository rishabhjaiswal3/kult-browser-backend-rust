use chrono::Utc;
use kult_browser_backend_rust::game::model::gameImageModel::{
    GameImages, ImageObject, IndexedImage, OrientedImage, OrientedImageArray,
};
use kult_browser_backend_rust::game::repository::gameImageRepository::GameModelImageRepository;
use kult_browser_backend_rust::mongo::connection;
use kult_browser_backend_rust::{GameModel, GameModelRepository};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;

#[tokio::test]
async fn test_game_repository_lifecycle() {
    // 1. Connect to DB
    let db = connection::connect()
        .await
        .expect("Failed to connect to Mongo");

    // 2. Initialize Repositories
    let game_repo = GameModelRepository::new(&db);
    let image_repo = GameModelImageRepository::new(&db);

    // 3. Prepare Test Data
    let test_id = format!("test-game-{}", ObjectId::new());
    let new_game = GameModel {
        id: ObjectId::new(),
        identification: test_id.clone(),
        name: "Test Game".to_string(),
        platform: "web".to_string(),
        url: "https://example.com/game".to_string(),
        images: GameImages::default(),
        slogan: Some("A test game".to_string()),
        about: Some("Description".to_string()),
        category: Some("Action".to_string()),
        tags: Some(vec!["test".to_string(), "rust".to_string()]),
        rating: Some(4.5),
        rating_count: Some(10),
        metadata: None,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };

    // 4. Test Create
    println!("Testing Create...");
    let created_id = game_repo
        .create(&new_game)
        .await
        .expect("Failed to create game");
    assert_eq!(created_id, new_game.id.to_hex());

    // 5. Test Exists
    println!("Testing Exists...");
    let exists = game_repo
        .exists(&test_id)
        .await
        .expect("Failed to check existence");
    assert!(exists);

    // 6. Test Find By Identification
    println!("Testing Find...");
    let found_game = game_repo
        .find_by_identification(&test_id)
        .await
        .expect("Failed to find game");
    assert!(found_game.is_some());
    let found_game = found_game.unwrap();
    assert_eq!(found_game.name, "Test Game");

    // 7. Test Patch
    println!("Testing Patch...");
    let update_doc = doc! {
        "$set": { "name": "Patched Game Name" }
    };
    let patched_game = game_repo
        .patch(&test_id, update_doc)
        .await
        .expect("Patch failed");
    assert!(patched_game.is_some());
    assert_eq!(patched_game.unwrap().name, "Patched Game Name");

    // 8. Test Replace
    println!("Testing Replace...");
    let mut replacement_game = new_game.clone();
    replacement_game.name = "Replaced Game".to_string();
    let replaced_game = game_repo
        .replace(&test_id, &replacement_game)
        .await
        .expect("Replace failed");
    assert!(replaced_game.is_some());
    assert_eq!(replaced_game.unwrap().name, "Replaced Game");

    // 9. Test Image Repository - Update Hero
    println!("Testing Image Update (Hero)...");
    let test_image = ImageObject {
        url: "http://img.com/hero.jpg".to_string(),
        width: Some(1920),
        height: Some(1080),
        ..Default::default()
    };
    let hero_images = OrientedImage {
        horizontal: test_image.clone(),
        vertical: test_image.clone(),
        ..Default::default()
    };

    let updated_hero = image_repo
        .update_hero(&test_id, &hero_images)
        .await
        .expect("Hero update failed");
    assert!(updated_hero.is_some());

    // Verify hero update
    let loaded_images = image_repo
        .get_images(&test_id)
        .await
        .expect("Get images failed")
        .unwrap();
    assert_eq!(loaded_images.hero.horizontal.url, "http://img.com/hero.jpg");

    // 10. Test Image Repository - Carousel
    println!("Testing Image Update (Carousel)...");
    let carousel_img = IndexedImage {
        index: 0,
        image: test_image.clone(),
    };
    let carousel = OrientedImageArray {
        horizontal: vec![carousel_img.clone()],
        vertical: vec![],
        ..Default::default()
    };
    image_repo
        .update_carousel(&test_id, &carousel)
        .await
        .expect("Carousel update failed");

    // 11. Test Add Carousel Image
    println!("Testing Add Carousel Image...");
    let new_carousel_img = IndexedImage {
        index: 1,
        image: ImageObject {
            url: "http://img.com/2.jpg".to_string(),
            ..Default::default()
        },
    };
    image_repo
        .add_carousel_image(&test_id, "horizontal", &new_carousel_img)
        .await
        .expect("Add carousel image failed");

    // 12. Test Remove Carousel Image
    println!("Testing Remove Carousel Image...");
    image_repo
        .remove_carousel_image(&test_id, "horizontal", 0)
        .await
        .expect("Remove carousel image failed");

    // 13. Test Delete
    println!("Testing Delete...");
    let deleted = game_repo.delete(&test_id).await.expect("Delete failed");
    assert!(deleted);

    let exists_after_delete = game_repo
        .exists(&test_id)
        .await
        .expect("Check existence failed");
    assert!(!exists_after_delete);

    println!("✅ All Repository Tests Passed!");
}
