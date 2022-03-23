use gotham::helpers::http::response::create_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::{single_middleware, single_pipeline};
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::StateData;
use gotham::state::{FromState, State};
use hyper::{body, Body, Response, StatusCode};

use futures::executor;
use image;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use crate::state;

// ---------------
// AI Gym REST API
// ---------------

#[derive(Clone, StateData)]
pub(crate) struct GothamState<T: 'static + Send + Sync + Clone + std::panic::RefUnwindSafe> {
    pub(crate) inner: Arc<Mutex<state::AIGymState<T>>>,
}

pub(crate) fn router<T: 'static + Send + Sync + Clone + std::panic::RefUnwindSafe>(
    state: GothamState<T>,
) -> Router {
    let middleware = StateMiddleware::new(state);
    let pipeline = single_middleware(middleware);

    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    build_router(chain, pipelines, |route| {
        route.get("/screen.png").to(screen::<T>);
        route.post("/step").to(step::<T>);
        route.post("/reset").to(reset::<T>);
    })
}

fn screen<T: 'static + Send + Sync + Clone + std::panic::RefUnwindSafe>(
    state: State,
) -> (State, Response<Body>) {
    let state_: &GothamState<T> = GothamState::borrow_from(&state);
    let state__ = state_.inner.lock().unwrap().clone();
    let image = state__.screen.clone().unwrap();

    let mut bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .unwrap();

    let response = create_response::<Vec<u8>>(&state, StatusCode::OK, mime::TEXT_PLAIN, bytes);

    return (state, response);
}
fn step<T: 'static + Send + Sync + Clone + std::panic::RefUnwindSafe>(
    mut state: State,
) -> (State, String) {
    let body_ = Body::take_from(&mut state);
    let valid_body = executor::block_on(body::to_bytes(body_)).unwrap();
    let action = String::from_utf8(valid_body.to_vec()).unwrap();

    let state_: &GothamState<T> = GothamState::borrow_from(&state);

    loop {
        let mut ai_gym_state = state_.inner.lock().unwrap();
        if ai_gym_state.__is_environment_paused {
            ai_gym_state.__action_unparsed_string = action;
            break;
        }
    }

    let mut reward = 0.0;
    let is_terminated;
    loop {
        let ai_gym_state = state_.inner.lock().unwrap();

        if ai_gym_state.__is_environment_paused {
            if ai_gym_state.rewards.len() > 0 {
                reward = ai_gym_state.rewards[ai_gym_state.rewards.len() - 1];
            }
            if ai_gym_state.rewards.len() > 1 {
                reward -= ai_gym_state.rewards[ai_gym_state.rewards.len() - 2];
            }

            is_terminated = ai_gym_state.is_terminated.clone();

            break;
        }
    }

    return (
        state,
        format!(
            "{{\"reward\": {}, \"is_terminated\": {}}}",
            reward, is_terminated
        ),
    );
}

fn reset<T: 'static + Send + Sync + Clone + std::panic::RefUnwindSafe>(
    state: State,
) -> (State, String) {
    {
        let state_: &GothamState<T> = GothamState::borrow_from(&state);
        let mut ai_gym_state = state_.inner.lock().unwrap();
        ai_gym_state.__request_for_reset = true;
    }
    return (state, "ok".to_string());
}
