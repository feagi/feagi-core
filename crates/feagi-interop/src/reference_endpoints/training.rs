//! Training: reinforcement learning signals (reward, punishment, shock) and fitness evaluation.
//! Converted from the old `training` endpoints (`/v1/training/*`).

use feagi_basis::feagi_basis_proc_common::define_request_message_category;

define_request_message_category! {
    category_name: "Training",
    base_path: "v1/training",
    category_description: "Reinforcement learning signals (reward, punishment, shock) and brain fitness evaluation",
    read: {
        "ShockOptions": {
            path: "shock/options",
            description: "Get available shock/punishment options for training",
            alloc_only: true,
            response: {
                options: Vec<String>, "Available shock options",
            },
        },
        "ShockStatus": {
            path: "shock/status",
            description: "Get current shock/punishment status and active scenarios",
            alloc_only: true,
            response: {
                active: bool, "Whether shock is active",
                scenarios: Vec<String>, "Active shock scenarios",
            },
        },
        "BrainFitness": {
            path: "brain_fitness",
            description: "Get current brain fitness score for evolutionary evaluation",
            response: {
                fitness: f64, "Current brain fitness score",
            },
        },
        "FitnessCriteria": {
            path: "fitness_criteria",
            description: "Get fitness evaluation criteria used for brain assessment",
            alloc_only: true,
            response: {
                criteria: serde_json::Value, "Fitness criteria object",
            },
        },
        "FitnessStats": {
            path: "fitness_stats",
            description: "Get fitness statistics including historical performance data",
            alloc_only: true,
            response: {
                stats: serde_json::Value, "Fitness statistics object",
            },
        },
        "TrainingReport": {
            path: "training_report",
            description: "Get training progress report with performance metrics and insights",
            alloc_only: true,
            response: {
                report: serde_json::Value, "Training report object",
            },
        },
        "Status": {
            path: "status",
            description: "Get training system status including active state and current mode",
            alloc_only: true,
            response: {
                active: bool, "Whether training is active",
                mode: String, "Current training mode (e.g. idle)",
            },
        },
        "Stats": {
            path: "stats",
            description: "Get training statistics including episodes and rewards",
            response: {
                total_episodes: u64, "Total training episodes",
                total_rewards: f64, "Total accumulated reward",
            },
        },
    },
    create: {
        "Shock": {
            path: "shock",
            description: "Configure shock/punishment scenarios for reinforcement learning",
            alloc_only: true,
            payload: {
                shock: serde_json::Value, "Shock scenario configuration",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "ShockActivate": {
            path: "shock/activate",
            description: "Activate shock/punishment scenario immediately",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "RewardIntensity": {
            path: "reward/intensity",
            description: "Set reward intensity for positive reinforcement",
            alloc_only: true,
            payload: {
                intensity: f64, "Reward intensity",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "PunishmentIntensity": {
            path: "punishment/intensity",
            description: "Set punishment intensity for negative reinforcement",
            alloc_only: true,
            payload: {
                intensity: f64, "Punishment intensity",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "Reward": {
            path: "reward",
            description: "Apply reward signal for positive reinforcement learning",
            alloc_only: true,
            payload: {
                reward: serde_json::Value, "Reward signal data",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "Punishment": {
            path: "punishment",
            description: "Apply punishment signal for negative reinforcement learning",
            alloc_only: true,
            payload: {
                punishment: serde_json::Value, "Punishment signal data",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "GameOver": {
            path: "gameover",
            description: "Signal game over condition for episode termination",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "FitnessCriteria": {
            path: "fitness_criteria",
            description: "Set fitness evaluation criteria (alternative to the edit endpoint)",
            alloc_only: true,
            payload: {
                criteria: serde_json::Value, "Fitness criteria object",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "Config": {
            path: "config",
            description: "Configure training parameters including learning rates and reward settings",
            alloc_only: true,
            payload: {
                config: serde_json::Value, "Training configuration object",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
    edit: {
        "FitnessCriteria": {
            path: "fitness_criteria",
            description: "Update fitness evaluation criteria for brain assessment",
            alloc_only: true,
            payload: {
                criteria: serde_json::Value, "Fitness criteria object",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
        "FitnessStats": {
            path: "fitness_stats",
            description: "Update fitness statistics with new data",
            alloc_only: true,
            payload: {
                stats: serde_json::Value, "Fitness statistics object",
            },
            response: {
                message: String, "Confirmation message",
            },
        },
    },
    delete: {
        "FitnessStats": {
            path: "fitness_stats",
            description: "Delete fitness statistics data",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
        "ResetFitnessStats": {
            path: "reset_fitness_stats",
            description: "Reset fitness statistics to initial state",
            alloc_only: true,
            response: {
                message: String, "Confirmation message",
            },
        },
    },
}

// NOTES:
// Every old handler was a TODO stub returning placeholder values; response fields come from those placeholders.
// All "message" responses were HashMap<String, String> {"message"}; wrapped as message field.
// Shock: only "shock" key was checked (presence only), shape unknown -> serde_json::Value.
// RewardIntensity/PunishmentIntensity: body ignored; "intensity: f64" payload is a guess.
// Reward/Punishment/FitnessCriteria/Config/FitnessStats(edit): bodies were ignored untyped maps; payload keys guessed, Values.
// FitnessCriteria/FitnessStats/TrainingReport reads: placeholders returned empty objects -> serde_json::Value.
// ShockOptions/ShockStatus: list element type unknown (empty placeholders), assumed String.
