# ApiV2010AccountValidationRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for the Caller ID. | [optional]
**call_sid** | Option<**String**> | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Caller ID is associated with. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**phone_number** | Option<**String**> | The phone number to verify in [E.164](https://www.twilio.com/docs/glossary/what-e164) format, which consists of a + followed by the country code and subscriber number. | [optional]
**validation_code** | Option<**String**> | The 6 digit validation code that someone must enter to validate the Caller ID  when `phone_number` is called. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


