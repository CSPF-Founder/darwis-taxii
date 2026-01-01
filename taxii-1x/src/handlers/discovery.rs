//! Discovery request handlers.

use crate::constants::{
    VID_TAXII_HTTP_10, VID_TAXII_SERVICES_10, VID_TAXII_SERVICES_11, VID_TAXII_XML_10,
    VID_TAXII_XML_11,
};
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::{tm10, tm11};
use taxii_db::Taxii1Repository;

use super::base::{HandlerContext, TaxiiHeaders, generate_id};

/// TAXII 1.1 Discovery Request Handler.
pub struct DiscoveryRequest11Handler;

impl DiscoveryRequest11Handler {
    /// Handle a TAXII 1.1 Discovery Request.
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        let request = match message {
            tm11::Taxii11Message::DiscoveryRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Discovery Request message",
                    None,
                ));
            }
        };

        let mut response = tm11::DiscoveryResponse::new(generate_id(), &request.message_id);

        // Get advertised services from database
        let services = ctx
            .persistence
            .get_advertised_services(&ctx.service.id)
            .await?;

        response.service_instances = services
            .into_iter()
            .map(|service| {
                let service_id = service.id.as_deref().unwrap_or_default();
                let address = service
                    .properties
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| format!("/services/{service_id}/"));

                let protocol_binding = service
                    .properties
                    .get("protocol_binding")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());

                let message_bindings: Vec<String> = service
                    .properties
                    .get("message_bindings")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_else(|| vec![VID_TAXII_XML_11.to_string()]);

                tm11::ServiceInstance::new(
                    service.service_type,
                    VID_TAXII_SERVICES_11,
                    protocol_binding,
                    address,
                    message_bindings,
                )
            })
            .collect();

        Ok(tm11::Taxii11Message::DiscoveryResponse(response))
    }
}

/// TAXII 1.0 Discovery Request Handler.
pub struct DiscoveryRequest10Handler;

impl DiscoveryRequest10Handler {
    /// Handle a TAXII 1.0 Discovery Request.
    pub async fn handle_10(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm10::Taxii10Message,
    ) -> Taxii1xResult<tm10::Taxii10Message> {
        let request = match message {
            tm10::Taxii10Message::DiscoveryRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Discovery Request message",
                    None,
                ));
            }
        };

        let mut response = tm10::DiscoveryResponse::new(generate_id(), &request.message_id);

        // Get advertised services from database
        let services = ctx
            .persistence
            .get_advertised_services(&ctx.service.id)
            .await?;

        response.service_instances = services
            .into_iter()
            .map(|service| {
                let service_id = service.id.as_deref().unwrap_or_default();
                let address = service
                    .properties
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| format!("/services/{service_id}/"));

                let protocol_binding = service
                    .properties
                    .get("protocol_binding")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());

                let message_bindings: Vec<String> = service
                    .properties
                    .get("message_bindings")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_else(|| vec![VID_TAXII_XML_10.to_string()]);

                tm10::ServiceInstance::new(
                    service.service_type,
                    VID_TAXII_SERVICES_10,
                    protocol_binding,
                    address,
                    message_bindings,
                )
            })
            .collect();

        Ok(tm10::Taxii10Message::DiscoveryResponse(response))
    }
}
