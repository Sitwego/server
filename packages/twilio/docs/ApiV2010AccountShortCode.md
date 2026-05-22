# ApiV2010AccountShortCode

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this ShortCode resource. | [optional]
**api_version** | Option<**String**> | The API version used to start a new TwiML session when an SMS message is sent to this short code. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that this resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that this resource was last updated, specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**friendly_name** | Option<**String**> | A string that you assigned to describe this resource. By default, the `FriendlyName` is the short code. | [optional]
**short_code** | Option<**String**> | The short code. e.g., 894546. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify this ShortCode resource. | [optional]
**sms_fallback_method** | Option<**SmsFallbackMethod**> | The HTTP method we use to call the `sms_fallback_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**sms_fallback_url** | Option<**String**> | The URL that we call if an error occurs while retrieving or executing the TwiML from `sms_url`. | [optional]
**sms_method** | Option<**SmsMethod**> | The HTTP method we use to call the `sms_url`. Can be: `GET` or `POST`. (enum: GET, POST) | [optional]
**sms_url** | Option<**String**> | The URL we call when receiving an incoming SMS message to this short code. | [optional]
**uri** | Option<**String**> | The URI of this resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


