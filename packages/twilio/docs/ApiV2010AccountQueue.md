# ApiV2010AccountQueue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_updated** | Option<**String**> | The date and time in GMT that this resource was last updated, specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**current_size** | Option<**i32**> | The number of calls currently in the queue. | [optional][default to 0]
**friendly_name** | Option<**String**> | A string that you assigned to describe this resource. | [optional]
**uri** | Option<**String**> | The URI of this resource, relative to `https://api.twilio.com`. | [optional]
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Queue resource. | [optional]
**average_wait_time** | Option<**i32**> |  The average wait time in seconds of the members in this queue. This is calculated at the time of the request. | [optional][default to 0]
**sid** | Option<**String**> | The unique string that that we created to identify this Queue resource. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that this resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**max_size** | Option<**i32**> |  The maximum number of calls that can be in the queue. The default is 1000 and the maximum is 5000. | [optional][default to 0]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


