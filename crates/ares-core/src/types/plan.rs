use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum PlanStatus {
    Draft,
    Generated,
    Simulated,
    Approved,
    Scheduled,
    Executing,
    Completed,
    Failed,
    Replanned,
    Cancelled,
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            PlanStatus::Draft => "Draft",
            PlanStatus::Generated => "Generated",
            PlanStatus::Simulated => "Simulated",
            PlanStatus::Approved => "Approved",
            PlanStatus::Scheduled => "Scheduled",
            PlanStatus::Executing => "Executing",
            PlanStatus::Completed => "Completed",
            PlanStatus::Failed => "Failed",
            PlanStatus::Replanned => "Replanned",
            PlanStatus::Cancelled => "Cancelled",
        };
        write!(f, "{}", val)
    }
}

impl std::str::FromStr for PlanStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(PlanStatus::Draft),
            "Generated" => Ok(PlanStatus::Generated),
            "Simulated" => Ok(PlanStatus::Simulated),
            "Approved" => Ok(PlanStatus::Approved),
            "Scheduled" => Ok(PlanStatus::Scheduled),
            "Executing" => Ok(PlanStatus::Executing),
            "Completed" => Ok(PlanStatus::Completed),
            "Failed" => Ok(PlanStatus::Failed),
            "Replanned" => Ok(PlanStatus::Replanned),
            "Cancelled" => Ok(PlanStatus::Cancelled),
            _ => Err(format!("Unknown plan status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::InProgress => "InProgress",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
        };
        write!(f, "{}", val)
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(TaskStatus::Pending),
            "InProgress" => Ok(TaskStatus::InProgress),
            "Completed" => Ok(TaskStatus::Completed),
            "Failed" => Ok(TaskStatus::Failed),
            _ => Err(format!("Unknown task status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Goal {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Plan {
    pub id: String,
    pub goal_id: String,
    pub state: PlanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Milestone {
    pub id: String,
    pub plan_id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct Task {
    pub id: String,
    pub milestone_id: Option<String>,
    pub plan_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub estimated_duration: Option<i32>, // in minutes
    pub complexity: Option<String>,      // "Low", "Medium", "High"
    pub execution_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct TaskDependency {
    pub task_id: String,
    pub depends_on_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct PlanDetails {
    pub plan: Plan,
    pub goal: Goal,
    pub milestones: Vec<Milestone>,
    pub tasks: Vec<Task>,
    pub dependencies: Vec<TaskDependency>,
}
