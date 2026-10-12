use crownos_protocols::agent_access::v1::client::crownos_agent_access_v1::{
    self, CrownosAgentAccessV1,
};
use sctk::{
    dispatch2::Dispatch2,
    reexports::client::{Connection, Proxy, QueueHandle, backend::ObjectId},
};

use super::CrownEvent;
use crate::wayland::{
    error::Error,
    event_loop::{QueuedEvent, WaylandPlatform},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentAccessId(ObjectId);

/// The compositor's answer to [`WaylandPlatform::create_agent_access_request`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentAccessDecision {
    /// Expose the UI to agents; present the token to the agent bridge.
    Granted(String),
    Denied,
    /// A grant ended; stop exposing the UI.
    Revoked,
}

impl From<AgentAccessDecision> for crate::AgentAccess {
    fn from(decision: AgentAccessDecision) -> Self {
        match decision {
            AgentAccessDecision::Granted(token) => Self::Granted(token),
            AgentAccessDecision::Denied => Self::Denied,
            AgentAccessDecision::Revoked => Self::Revoked,
        }
    }
}

/// A pending request or a live grant. Dropping it withdraws the request or ends the grant.
#[derive(Debug)]
pub struct AgentAccessRequest {
    proxy: CrownosAgentAccessV1,
}

impl WaylandPlatform {
    /// Asks the compositor, and through it the user, to let this app expose its UI to
    /// AI agents. The answer arrives as [`CrownEvent::AgentAccess`], possibly much later.
    /// An empty `app_id` is always denied.
    pub fn create_agent_access_request(
        &self,
        app_id: &str,
        reason: &str,
    ) -> Result<AgentAccessRequest, Error> {
        let manager = self
            .crown
            .agent_access
            .as_ref()
            .ok_or(Error::MissingGlobal("crownos_agent_access_manager_v1"))?;
        let proxy = manager.request_access(
            app_id.to_owned(),
            reason.to_owned(),
            &self.queue_handle,
            AgentAccessData,
        );
        Ok(AgentAccessRequest { proxy })
    }
}

impl AgentAccessRequest {
    pub fn id(&self) -> AgentAccessId {
        AgentAccessId(self.proxy.id())
    }
}

impl Drop for AgentAccessRequest {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

pub(crate) struct AgentAccessData;

impl Dispatch2<CrownosAgentAccessV1, WaylandPlatform> for AgentAccessData {
    fn event(
        &self,
        state: &mut WaylandPlatform,
        request: &CrownosAgentAccessV1,
        event: crownos_agent_access_v1::Event,
        _: &Connection,
        _: &QueueHandle<WaylandPlatform>,
    ) {
        let decision = match event {
            crownos_agent_access_v1::Event::Granted { token } => {
                AgentAccessDecision::Granted(token)
            }
            crownos_agent_access_v1::Event::Denied => AgentAccessDecision::Denied,
            crownos_agent_access_v1::Event::Revoked => AgentAccessDecision::Revoked,
            _ => return,
        };
        state
            .pending_events
            .push(QueuedEvent::Crown(CrownEvent::AgentAccess {
                request: AgentAccessId(request.id()),
                decision,
            }));
    }
}
