use super::*;

fn answers_from_prompt(prompt: &crate::tui::QuestionPromptState) -> serde_json::Value {
    let questions = prompt.questions.as_array().cloned().unwrap_or_default();
    serde_json::Value::Object(
        questions
            .iter()
            .filter_map(|question| {
                let id = question.get("id")?.as_str()?;
                let mut answer = serde_json::Map::new();
                answer.insert(
                    "option_ids".into(),
                    serde_json::json!(prompt.selected.get(id).cloned().unwrap_or_default()),
                );
                if let Some(text) = prompt.free_text.get(id)
                    && !text.trim().is_empty()
                {
                    answer.insert("free_text".into(), serde_json::json!(text));
                }
                Some((id.to_string(), serde_json::Value::Object(answer)))
            })
            .collect(),
    )
}

pub(super) async fn handle_question_prompt_key(
    app: &mut App,
    code: KeyCode,
    text_input: Option<String>,
    remote: &mut RemoteConnection,
) -> Result<()> {
    let Some(prompt) = app.question_prompt.as_mut() else {
        return Ok(());
    };

    if prompt.reviewing {
        match code {
            KeyCode::Esc => {
                let request_id = prompt.request_id.clone();
                app.question_prompt = None;
                remote.send_question_cancel(&request_id).await?;
            }
            KeyCode::Left | KeyCode::Backspace => {
                prompt.reviewing = false;
                prompt.question_index = prompt
                    .questions
                    .as_array()
                    .map_or(0, |q| q.len().saturating_sub(1));
                prompt.option_index = 0;
            }
            KeyCode::Enter => {
                let request_id = prompt.request_id.clone();
                let answers = answers_from_prompt(prompt);
                app.question_prompt = None;
                remote.send_question_response(&request_id, answers).await?;
            }
            _ => {}
        }
        return Ok(());
    }

    let questions = prompt.questions.as_array().cloned().unwrap_or_default();
    let Some(question) = questions.get(prompt.question_index) else {
        app.question_prompt = None;
        return Ok(());
    };
    let qid = question
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let options = question
        .get("options")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let multi = question
        .get("multi_select")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if prompt.editing_other {
        match code {
            KeyCode::Enter | KeyCode::Esc => prompt.editing_other = false,
            KeyCode::Backspace => {
                prompt.free_text.entry(qid).or_default().pop();
            }
            KeyCode::Char(_) => {
                if let Some(text) = text_input {
                    let value = prompt.free_text.entry(qid).or_default();
                    if value.chars().count() + text.chars().count() <= 4_000 {
                        value.push_str(&text);
                    }
                }
            }
            _ => {}
        }
        return Ok(());
    }

    match code {
        KeyCode::Esc => {
            let request_id = prompt.request_id.clone();
            app.question_prompt = None;
            remote.send_question_cancel(&request_id).await?;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            prompt.option_index = prompt.option_index.saturating_sub(1)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            prompt.option_index = (prompt.option_index + 1).min(options.len().saturating_sub(1))
        }
        KeyCode::Left | KeyCode::Backspace if prompt.question_index > 0 => {
            prompt.question_index -= 1;
            prompt.option_index = 0;
        }
        KeyCode::Char('o') => prompt.editing_other = true,
        KeyCode::Char(' ') if multi => {
            if let Some(id) = options
                .get(prompt.option_index)
                .and_then(|o| o.get("id"))
                .and_then(serde_json::Value::as_str)
            {
                let selected = prompt.selected.entry(qid.clone()).or_default();
                if selected.iter().any(|value| value == id) {
                    selected.retain(|value| value != id);
                } else {
                    selected.push(id.to_string());
                }
            }
        }
        KeyCode::Enter => {
            if let Some(id) = options
                .get(prompt.option_index)
                .and_then(|o| o.get("id"))
                .and_then(serde_json::Value::as_str)
            {
                let selected = prompt.selected.entry(qid.clone()).or_default();
                if !multi {
                    selected.clear();
                }
                if !selected.iter().any(|value| value == id) {
                    selected.push(id.to_string());
                }
            }
            let has_answer = prompt.selected.get(&qid).is_some_and(|v| !v.is_empty())
                || prompt
                    .free_text
                    .get(&qid)
                    .is_some_and(|v| !v.trim().is_empty());
            if !has_answer {
                app.set_status_notice("Select an option or press 'o' to enter Other");
                return Ok(());
            }
            if prompt.question_index + 1 < questions.len() {
                prompt.question_index += 1;
                prompt.option_index = 0;
            } else if (0..questions.len()).all(|index| {
                questions[index]
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|id| {
                        prompt.selected.get(id).is_some_and(|v| !v.is_empty())
                            || prompt
                                .free_text
                                .get(id)
                                .is_some_and(|v| !v.trim().is_empty())
                    })
            }) {
                prompt.reviewing = true;
            }
        }
        _ => {}
    }
    Ok(())
}
