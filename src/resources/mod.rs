pub mod authentik;
pub mod authentik_application;
pub mod authentik_group;
pub mod authentik_provider_oauth;
pub mod authentik_user;

pub use authentik::Manager as AuthentikManager;
pub use authentik_application::Manager as AuthentikAppManager;
pub use authentik_group::Manager as AuthentikGroupManager;
pub use authentik_provider_oauth::Manager as AuthentikOAuthManager;
pub use authentik_user::Manager as AuthentikUserManager;
