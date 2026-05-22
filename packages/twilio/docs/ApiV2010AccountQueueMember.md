# ApiV2010AccountQueueMember

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**call_sid** | Option<**String**> | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Member resource is associated with. | [optional]
**date_enqueued** | Option<**String**> | The date that the member was enqueued, given in RFC 2822 format. | [optional]
**position** | Option<**i32**> | This member's current position in the queue. | [optional][default to 0]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**wait_time** | Option<**i32**> | The number of seconds the member has been in the queue. | [optional][default to 0]
**queue_sid** | Option<**String**> | The SID of the Queue the member is in. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


