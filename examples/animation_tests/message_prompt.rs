#![allow(unused)]

use crate::result_traits::*;
use ascii_engine::prelude::*;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct MessagePrompt {
  hash: u64,
  /// The time before the message can be shown again.
  display_cooldown: Duration,
  /// A timestamp for the last time the message was shown.
  display_time: Option<Instant>,
}

impl MessagePrompt {
  pub const NAME: &str = "message_prompt";
  /// If the prompt has only 1 message, this will activate it.
  // TODO: Maybe think of a way to incorporate multiple messages.
  pub const ANIMATION_ACTIVATE: &str = "activate";
  const AIR_CHARACTER: char = ';';

  pub fn create_all(model_manager: &mut ModelManager) -> Vec<MessagePrompt> {
    let basic_message_string = "Hello World".to_string();
    let basic_prompt_model = Self::new_model((20, 10), basic_message_string, None, None);
    log::debug!("{:#?}", basic_prompt_model);
    let basic_message = MessagePrompt {
      hash: basic_prompt_model.get_hash(),
      display_cooldown: Duration::from_secs(1),
      display_time: None,
    };

    model_manager.add_models_to_world(vec![basic_prompt_model]);

    vec![basic_message]
  }

  fn new_model(
    position: (usize, usize),
    message: String,
    appearance_override: Option<Sprite>,
    hitbox_override: Option<Rectangle>,
  ) -> ModelData {
    let base_appearance = "c";
    let anchor_character = 'c';
    let anchor_replacement_character = 'x';
    let base_appearance = Sprite::new(
      base_appearance,
      anchor_character,
      anchor_replacement_character,
      Self::AIR_CHARACTER,
    )
    .unwrap();
    let hitbox = Hitbox::new(Rectangle::new(1, 1), 0);
    let mut model = ModelData::new(
      position,
      base_appearance,
      hitbox,
      Strata(99),
      Self::NAME.to_string(),
    )
    .unwrap();

    Self::apply_animation(&mut model, message);
    Self::apply_tags(&mut model);

    model
  }

  fn apply_animation(model: &mut ModelData, message: String) {
    let frame_duration = 2000 / CONFIG.tick_duration;

    let message_box = Self::create_box_from_message(message);
    let anchor_character = 'c';
    let anchor_replacement_character = '/';
    log::debug!("Message: \n{:?}", message_box);
    let appearance = Sprite::new(
      message_box,
      anchor_character,
      anchor_replacement_character,
      Self::AIR_CHARACTER,
    )
    .unwrap();

    let loop_count = AnimationLoopCount::Limited(1);
    let animation = AnimationFrames::new(
      vec![AnimationFrame::new(appearance, frame_duration)],
      loop_count,
      None,
    );

    let model_appearance = model.get_appearance_data();
    let mut model_appearance = model_appearance.lock().unwrap();

    model_appearance.add_animation_to_model(Self::ANIMATION_ACTIVATE.into(), animation);
  }

  /// Returns the message surrounded in a box.
  ///
  /// # Example
  ///
  /// ```no_run,bash
  /// ┌──────────────┐
  /// │ Hello World! │
  /// └──────────────┘
  /// ```
  ///
  fn create_box_from_message(message: String) -> String {
    let width_border = 1;
    let message_size = message.chars().count();

    // ┌┐ └┘ ─│
    // let top = format!("c{}┐", "─".repeat(message_size + (width_border * 2)));
    // let bottom = format!("└{}┘", "─".repeat(message_size + (width_border * 2)));
    // let message_holder = format!(
    //   "│{border}{message}{border}│",
    // /\ \/ -|
    let top = format!("c{}\\", "-".repeat(message_size + (width_border * 2)));
    let bottom = format!("\\{}//", "-".repeat(message_size + (width_border * 2)));
    let message_holder = format!(
      "|{border}{message}{border}|",
      border = Self::AIR_CHARACTER
        .to_string()
        .as_str()
        .repeat(width_border)
    );

    format!(
      "{top}\n\
     {message_holder}\n\
     {bottom}"
    )
  }

  fn apply_tags(model: &mut ModelData) {
    model.add_tags(vec![Self::NAME.into(), Self::ANIMATION_ACTIVATE.into()]);
  }

  pub fn act_on_collision(
    collision_event: &ModelCollisions,
    model_manager: &mut ModelManager,
    existing_messages: &mut [Self],
  ) {
    existing_messages.iter_mut().for_each(|message_prompt| {
      if collision_event.contains_model(&message_prompt.hash) {
        if let Some(last_display) = message_prompt.display_time {
          if last_display.elapsed() >= message_prompt.display_cooldown {
            message_prompt.display_time = None;
          } else {
            return;
          }
        }

        model_manager
          .queue_model_animation(&message_prompt.hash, Self::ANIMATION_ACTIVATE, false)
          .log_if_err();
      }
    })
  }
}
