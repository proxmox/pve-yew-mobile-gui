mod page_dashboard;
pub use page_dashboard::PageDashboard;

mod page_resources;
pub use page_resources::{PageResources, ResourceFilter};

mod page_vm_status;
pub use page_vm_status::PageVmStatus;

mod page_lxc_status;
pub use page_lxc_status::PageLxcStatus;

mod page_node_status;
pub use page_node_status::PageNodeStatus;

mod page_storage_status;
pub use page_storage_status::PageStorageStatus;

mod page_login;
pub use page_login::PageLogin;

mod page_qemu_tasks;
pub use page_qemu_tasks::PageQemuTasks;

mod page_lxc_tasks;
pub use page_lxc_tasks::PageLxcTasks;

mod page_node_tasks;
pub use page_node_tasks::PageNodeTasks;

mod page_task_status;
pub use page_task_status::PageTaskStatus;

// mod page_logs;
// pub use page_logs::PageLogs;

mod page_settings;
pub use page_settings::PageSettings;

mod page_configuartion;
pub use page_configuartion::PageConfiguration;

mod page_not_found;
pub use page_not_found::PageNotFound;
