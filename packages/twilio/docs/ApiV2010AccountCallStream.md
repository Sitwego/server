# ApiV2010AccountCallStream

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**sid** | Option<**String**> | The SID of the Stream resource. | [optional]
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Stream resource. | [optional]
**call_sid** | Option<**String**> | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Stream resource is associated with. | [optional]
**name** | Option<**String**> | The user-specified name of this Stream, if one was given when the Stream was created. This can be used to stop the Stream. | [optional]
**status** | Option<[**models::StreamEnumStatus**](StreamEnumStatus.md)> |  | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that this resource was last updated, specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


