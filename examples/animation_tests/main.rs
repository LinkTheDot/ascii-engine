use animation_actor::*;
use ascii_engine::prelude::*;
use result_traits::*;

mod animation_actor;
mod message_prompt;
mod player;
mod result_traits;
mod world;

fn main() {
  let (mut screen, player) = world::initialize();
  let mut model_manager = screen.get_model_manager();
  let event_sync = screen.get_event_sync();
  let (user_input, input_kill_sender) = spawn_input_thread();

  loop {
    if let Ok(user_input) = user_input.try_recv() {
      match user_input.trim() {
        "q" => break,
        "w" | "a" | "s" | "d" => player.movement(&mut model_manager, &user_input),
        "l" => log::debug!("{:#?}", model_manager),
        "*" => world::save_world(&screen),
        "-" => world::delete_world_save(),
        _ => (),
      }
    }

    check_collision_events(&mut model_manager);

    screen.print_screen().log_if_err();

    event_sync.wait_for_tick().unwrap();
  }

  let _ = input_kill_sender.send(());
}

fn check_collision_events(model_manager: &mut ModelManager) {
  let collision_events = model_manager.take_collision_events();

  collision_events
    .into_iter()
    .for_each(|(_, collision_event)| {
      check_animation_actors(&collision_event, model_manager);
    });
}

fn check_animation_actors(collision_event: &ModelCollisions, model_manager: &mut ModelManager) {
  let animation_actors = model_manager.get_models_with_tags(vec![AnimationActor::NAME]);

  if let Some(animation_actor) = animation_actors
    .iter()
    .find(|actor_hash| collision_event.collision_list.contains(actor_hash))
  {
    AnimationActor::activate_animation(animation_actor, model_manager);
  }
}
