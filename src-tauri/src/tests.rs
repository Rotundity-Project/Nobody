#[test]
fn test_module_smoke() {
    assert!(true);
}

#[cfg(test)]
mod perf_benchmarks {
    use crate::models::{
        CharacterStats, CultivationRealm, Element, Grade, Lifespan, SpiritualRoot,
    };
    use crate::numerical_system::{Action, ActionResult, Context};
    use crate::plot_engine::{
        ActionType, PlayerAction, PlayerOption, PlotEngine, PlotState, Scene,
    };
    use std::hint::black_box;
    use std::time::Instant;

    fn sample_character() -> CharacterStats {
        CharacterStats {
            spiritual_root: SpiritualRoot {
                element: Element::Fire,
                elements: vec![Element::Fire],
                grade: Grade::Heavenly,
                affinity: 0.85,
            },
            cultivation_realm: CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            techniques: vec![],
            lifespan: Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 120,
        }
    }

    fn percentile_ms(samples: &[f64], p: f64) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    fn sample_batch_per_op_ms<F>(outer_rounds: usize, inner_iters: usize, mut f: F) -> Vec<f64>
    where
        F: FnMut(),
    {
        let mut samples = Vec::with_capacity(outer_rounds);
        for _ in 0..outer_rounds {
            let start = Instant::now();
            for _ in 0..inner_iters {
                f();
            }
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            samples.push(elapsed_ms / inner_iters as f64);
        }
        samples
    }

    #[test]
    #[ignore = "manual perf benchmark"]
    fn perf_plot_advance_p95_under_target() {
        let engine = PlotEngine::new();
        let mut scene = Scene::new(
            "perf_scene".to_string(),
            "Perf Scene".to_string(),
            "A performance benchmark scene".to_string(),
            "sect".to_string(),
        );
        scene.add_option(PlayerOption {
            id: 0,
            description: "Cultivate".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });
        let state = PlotState::new(scene);
        let action_result = ActionResult {
            success: true,
            description: "Player cultivated steadily.".to_string(),
            stat_changes: vec![],
            events: vec!["cultivation tick".to_string()],
        };

        let samples = sample_batch_per_op_ms(120, 200, || {
            let update = engine.advance_plot(&state, &action_result);
            black_box(update);
        });

        let p50 = percentile_ms(&samples, 0.50);
        let p95 = percentile_ms(&samples, 0.95);
        let p99 = percentile_ms(&samples, 0.99);
        println!(
            "PERF plot_advance per_op_ms p50={:.3} p95={:.3} p99={:.3}",
            p50, p95, p99
        );
        assert!(
            p95 < 2500.0,
            "plot advance p95 exceeds target: {:.3}ms",
            p95
        );
    }

    #[test]
    #[ignore = "manual perf benchmark"]
    fn perf_combat_parse_p95_under_target() {
        let engine = PlotEngine::new();
        let character = sample_character();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "day".to_string(),
            weather: None,
        };
        let options = vec![PlayerOption {
            id: 0,
            description: "Duel".to_string(),
            requirements: vec![],
            action: Action::Combat {
                target_id: "npc_rival".to_string(),
            },
        }];
        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "duel".to_string(),
            selected_option_id: Some(0),
            meta: None,
        };

        let samples = sample_batch_per_op_ms(160, 240, || {
            let parsed = engine
                .process_player_action(&action, &character, &options, &context)
                .expect("combat parse should succeed");
            black_box(parsed);
        });

        let p50 = percentile_ms(&samples, 0.50);
        let p95 = percentile_ms(&samples, 0.95);
        let p99 = percentile_ms(&samples, 0.99);
        println!(
            "PERF combat_parse per_op_ms p50={:.3} p95={:.3} p99={:.3}",
            p50, p95, p99
        );
        assert!(
            p95 < 1000.0,
            "combat parse p95 exceeds target: {:.3}ms",
            p95
        );
    }
}
