use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::handler::{ApiResponse, AppError};
use crate::middleware::AuthPlayer;
use crate::moments::comments::dto::{
    CreateCommentRequest, DeleteCommentResponse, UpdateCommentRequest,
};
use crate::moments::comments::service::CommentsService;

#[derive(Clone)]
pub struct CommentsState {
    pub service: CommentsService,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query, rename_all = "camelCase")]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

#[utoipa::path(
    post,
    path = "/api/moments/{moment_id}/comments",
    summary = "Create a comment",
    description = "Creates a top-level comment on a moment.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 200, description = "Created comment", body = crate::openapi::CommentApiResponse),
        (status = 400, description = "Invalid comment payload", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Comments"
)]
pub async fn create_comment(
    State(state): State<CommentsState>,
    auth: AuthPlayer,
    Path(moment_id): Path<String>,
    payload: Result<Json<CreateCommentRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .service
        .create_comment(&moment_id, &auth.wallet_address, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/moments/{moment_id}/comments",
    summary = "List comments",
    description = "Returns paginated top-level comments for a moment.",
    params(
        ("moment_id" = String, Path, description = "Shareable moment identifier"),
        PaginationQuery
    ),
    responses(
        (status = 200, description = "Moment comments", body = crate::openapi::CommentListApiResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Comments"
)]
pub async fn list_comments(
    State(state): State<CommentsState>,
    Path(moment_id): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Response {
    let page = query.page.unwrap_or_else(default_page);
    let per_page = query.per_page.unwrap_or_else(default_per_page);

    match state.service.list_comments(&moment_id, page, per_page).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/moments/comments/{comment_id}/replies",
    summary = "Create a reply",
    description = "Creates a reply to a top-level comment.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("comment_id" = String, Path, description = "Comment identifier")
    ),
    request_body = CreateCommentRequest,
    responses(
        (status = 200, description = "Created reply", body = crate::openapi::CommentApiResponse),
        (status = 400, description = "Invalid reply payload", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Comment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Comments"
)]
pub async fn create_reply(
    State(state): State<CommentsState>,
    auth: AuthPlayer,
    Path(comment_id): Path<String>,
    payload: Result<Json<CreateCommentRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .service
        .create_reply(&comment_id, &auth.wallet_address, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/moments/comments/{comment_id}/replies",
    summary = "List replies",
    description = "Returns paginated replies for a top-level comment.",
    params(
        ("comment_id" = String, Path, description = "Comment identifier"),
        PaginationQuery
    ),
    responses(
        (status = 200, description = "Comment replies", body = crate::openapi::CommentListApiResponse),
        (status = 400, description = "Replies can only be listed for top-level comments", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Comment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Comments"
)]
pub async fn list_replies(
    State(state): State<CommentsState>,
    Path(comment_id): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> Response {
    let page = query.page.unwrap_or_else(default_page);
    let per_page = query.per_page.unwrap_or_else(default_per_page);

    match state.service.list_replies(&comment_id, page, per_page).await {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/api/moments/comments/{comment_id}",
    summary = "Update a comment",
    description = "Updates a comment owned by the authenticated player.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("comment_id" = String, Path, description = "Comment identifier")
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Updated comment", body = crate::openapi::CommentApiResponse),
        (status = 400, description = "Invalid update payload", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 403, description = "Comment is not owned by the authenticated player", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Comment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Comments"
)]
pub async fn update_comment(
    State(state): State<CommentsState>,
    auth: AuthPlayer,
    Path(comment_id): Path<String>,
    payload: Result<Json<UpdateCommentRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .service
        .update_comment(&auth.wallet_address, &comment_id, request)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/moments/comments/{comment_id}",
    summary = "Delete a comment",
    description = "Deletes a comment or reply owned by the authenticated player.",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("comment_id" = String, Path, description = "Comment identifier")
    ),
    responses(
        (status = 200, description = "Deleted comment", body = crate::openapi::DeleteCommentApiResponse),
        (status = 400, description = "Comment already deleted or invalid request", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 403, description = "Comment is not owned by the authenticated player", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Comment not found", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Comments"
)]
pub async fn delete_comment(
    State(state): State<CommentsState>,
    auth: AuthPlayer,
    Path(comment_id): Path<String>,
) -> Response {
    match state
        .service
        .delete_comment(&auth.wallet_address, &comment_id)
        .await
    {
        Ok(_) => ApiResponse::success(DeleteCommentResponse {
            message: "Comment deleted successfully".to_string(),
        })
        .into_response(),
        Err(e) => e.into_response(),
    }
}

