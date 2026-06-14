use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

impl ToString for PlanStatus {
    fn to_string(&self) -> String {
        match self {
            PlanStatus::Draft => "Draft".to_string(),
            PlanStatus::Generated => "Generated".to_string(),
            PlanStatus::Simulated => "Simulated".to_string(),
            PlanStatus::Approved => "Approved".to_string(),
            PlanStatus::Scheduled => "Scheduled".to_string(),
            PlanStatus::Executing => "Executing".to_string(),
            PlanStatus::Completed => "Completed".to_string(),
            PlanStatus::Failed => "Failed".to_string(),
            PlanStatus::Replanned => "Replanned".to_string(),
            PlanStatus::Cancelled => "Cancelled".to_string(),
        }
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

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Pending => "Pending".to_string(),
            TaskStatus::InProgress => "InProgress".to_string(),
            TaskStatus::Completed => "Completed".to_string(),
            TaskStatus::Failed => "Failed".to_string(),
        }
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
    pub complexity: Option<String>, // "Low", "Medium", "High"
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
