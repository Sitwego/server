# ApiV2010AccountSipSipDomain

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the SipDomain resource. | [optional]
**api_version** | Option<**String**> | The API version used to process the call. | [optional]
**auth_type** | Option<**String**> | The types of authentication you have mapped to your domain. Can be: `IP_ACL` and `CREDENTIAL_LIST`. If you have both defined for your domain, both will be returned in a comma delimited string. If `auth_type` is not defined, the domain will not be able to receive any traffic. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**domain_name** | Option<**String**> | The unique address you reserve on Twilio to which you route your SIP traffic. Domain names can contain letters, digits, and \"-\" and must end with `sip.twilio.com`. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the SipDomain resource. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**voice_fallback_method** | Option<**VoiceFallbackMethod**> | The HTTP method we use to call `voice_fallback_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_fallback_url** | Option<**String**> | The URL that we call when an error occurs while retrieving or executing the TwiML requested from `voice_url`. | [optional]
**voice_method** | Option<**VoiceMethod**> | The HTTP method we use to call `voice_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_status_callback_method** | Option<**VoiceStatusCallbackMethod**> | The HTTP method we use to call `voice_status_callback_url`. Either `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_status_callback_url** | Option<**String**> | The URL that we call to pass status parameters (such as call ended) to your application. | [optional]
**voice_url** | Option<**String**> | The URL we call using the `voice_method` when the domain receives a call. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of mapping resources associated with the SIP Domain resource identified by their relative URIs. | [optional]
**sip_registration** | Option<**bool**> | Whether to allow SIP Endpoints to register with the domain to receive calls. | [optional]
**emergency_calling_enabled** | Option<**bool**> | Whether emergency calling is enabled for the domain. If enabled, allows emergency calls on the domain from phone numbers with validated addresses. | [optional]
**secure** | Option<**bool**> | Whether secure SIP is enabled for the domain. If enabled, TLS will be enforced and SRTP will be negotiated on all incoming calls to this sip domain. | [optional]
**byoc_trunk_sid** | Option<**String**> | The SID of the BYOC Trunk(Bring Your Own Carrier) resource that the Sip Domain will be associated with. | [optional]
**emergency_caller_sid** | Option<**String**> | Whether an emergency caller sid is configured for the domain. If present, this phone number will be used as the callback for the emergency call. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


