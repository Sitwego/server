# ApiV2010AccountIncomingPhoneNumber

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this IncomingPhoneNumber resource. | [optional]
**address_sid** | Option<**String**> | The SID of the Address resource associated with the phone number. | [optional]
**address_requirements** | Option<[**models::IncomingPhoneNumberEnumAddressRequirement**](IncomingPhoneNumberEnumAddressRequirement.md)> |  | [optional]
**api_version** | Option<**String**> | The API version used to start a new TwiML session. | [optional]
**beta** | Option<**bool**> | Whether the phone number is new to the Twilio platform. Can be: `true` or `false`. | [optional]
**capabilities** | Option<[**models::ApiV2010AccountIncomingPhoneNumberCapabilities**](ApiV2010AccountIncomingPhoneNumberCapabilities.md)> |  | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**identity_sid** | Option<**String**> | The SID of the Identity resource that we associate with the phone number. Some regions require an Identity to meet local regulations. | [optional]
**phone_number** | Option<**String**> | The phone number in [E.164](https://www.twilio.com/docs/glossary/what-e164) format, which consists of a + followed by the country code and subscriber number. | [optional]
**origin** | Option<**String**> | The phone number's origin. `twilio` identifies Twilio-owned phone numbers and `hosted` identifies hosted phone numbers. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify this IncomingPhoneNumber resource. | [optional]
**sms_application_sid** | Option<**String**> | The SID of the application that handles SMS messages sent to the phone number. If an `sms_application_sid` is present, we ignore all `sms_*_url` values and use those of the application. | [optional]
**sms_fallback_method** | Option<**SmsFallbackMethod**> | The HTTP method we use to call `sms_fallback_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**sms_fallback_url** | Option<**String**> | The URL that we call when an error occurs while retrieving or executing the TwiML from `sms_url`. | [optional]
**sms_method** | Option<**SmsMethod**> | The HTTP method we use to call `sms_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**sms_url** | Option<**String**> | The URL we call when the phone number receives an incoming SMS message. | [optional]
**status_callback** | Option<**String**> | The URL we call using the `status_callback_method` to send status information to your application. | [optional]
**status_callback_method** | Option<**StatusCallbackMethod**> | The HTTP method we use to call `status_callback`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**trunk_sid** | Option<**String**> | The SID of the Trunk that handles calls to the phone number. If a `trunk_sid` is present, we ignore all of the voice urls and voice applications and use those set on the Trunk. Setting a `trunk_sid` will automatically delete your `voice_application_sid` and vice versa. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**voice_receive_mode** | Option<[**models::IncomingPhoneNumberEnumVoiceReceiveMode**](IncomingPhoneNumberEnumVoiceReceiveMode.md)> |  | [optional]
**voice_application_sid** | Option<**String**> | The SID of the application that handles calls to the phone number. If a `voice_application_sid` is present, we ignore all of the voice urls and use those set on the application. Setting a `voice_application_sid` will automatically delete your `trunk_sid` and vice versa. | [optional]
**voice_caller_id_lookup** | Option<**bool**> | Whether we look up the caller's caller-ID name from the CNAM database ($0.01 per look up). Can be: `true` or `false`. | [optional]
**voice_fallback_method** | Option<**VoiceFallbackMethod**> | The HTTP method we use to call `voice_fallback_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_fallback_url** | Option<**String**> | The URL that we call when an error occurs retrieving or executing the TwiML requested by `url`. | [optional]
**voice_method** | Option<**VoiceMethod**> | The HTTP method we use to call `voice_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_url** | Option<**String**> | The URL we call when the phone number receives a call. The `voice_url` will not be used if a `voice_application_sid` or a `trunk_sid` is set. | [optional]
**emergency_status** | Option<[**models::IncomingPhoneNumberEnumEmergencyStatus**](IncomingPhoneNumberEnumEmergencyStatus.md)> |  | [optional]
**emergency_address_sid** | Option<**String**> | The SID of the emergency address configuration that we use for emergency calling from this phone number. | [optional]
**emergency_address_status** | Option<[**models::IncomingPhoneNumberEnumEmergencyAddressStatus**](IncomingPhoneNumberEnumEmergencyAddressStatus.md)> |  | [optional]
**bundle_sid** | Option<**String**> | The SID of the Bundle resource that you associate with the phone number. Some regions require a Bundle to meet local Regulations. | [optional]
**status** | Option<**String**> |  | [optional]
**r#type** | Option<**String**> | The phone number type. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


