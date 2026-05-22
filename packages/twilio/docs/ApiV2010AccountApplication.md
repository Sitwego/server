# ApiV2010AccountApplication

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Application resource. | [optional]
**api_version** | Option<**String**> | The API version used to start a new TwiML session. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**message_status_callback** | Option<**String**> | The URL we call using a POST method to send message status information to your application. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the Application resource. | [optional]
**sms_fallback_method** | Option<**SmsFallbackMethod**> | The HTTP method we use to call `sms_fallback_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**sms_fallback_url** | Option<**String**> | The URL that we call when an error occurs while retrieving or executing the TwiML from `sms_url`. | [optional]
**sms_method** | Option<**SmsMethod**> | The HTTP method we use to call `sms_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**sms_status_callback** | Option<**String**> | The URL we call using a POST method to send status information to your application about SMS messages that refer to the application. | [optional]
**sms_url** | Option<**String**> | The URL we call when the phone number receives an incoming SMS message. | [optional]
**status_callback** | Option<**String**> | The URL we call using the `status_callback_method` to send status information to your application. | [optional]
**status_callback_method** | Option<**StatusCallbackMethod**> | The HTTP method we use to call `status_callback`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**voice_caller_id_lookup** | Option<**bool**> | Whether we look up the caller's caller-ID name from the CNAM database (additional charges apply). Can be: `true` or `false`. | [optional]
**voice_fallback_method** | Option<**VoiceFallbackMethod**> | The HTTP method we use to call `voice_fallback_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_fallback_url** | Option<**String**> | The URL that we call when an error occurs retrieving or executing the TwiML requested by `url`. | [optional]
**voice_method** | Option<**VoiceMethod**> | The HTTP method we use to call `voice_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**voice_url** | Option<**String**> | The URL we call when the phone number assigned to this application receives a call. | [optional]
**public_application_connect_enabled** | Option<**bool**> | Whether to allow other Twilio accounts to dial this applicaton using Dial verb. Can be: `true` or `false`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


