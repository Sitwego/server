# ApiV2010AccountRecordingRecordingAddOnResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**sid** | Option<**String**> | The unique string that that we created to identify the Recording AddOnResult resource. | [optional]
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult resource. | [optional]
**status** | Option<[**models::RecordingAddOnResultEnumStatus**](RecordingAddOnResultEnumStatus.md)> |  | [optional]
**add_on_sid** | Option<**String**> | The SID of the Add-on to which the result belongs. | [optional]
**add_on_configuration_sid** | Option<**String**> | The SID of the Add-on configuration. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_completed** | Option<**String**> | The date and time in GMT that the result was completed specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**reference_sid** | Option<**String**> | The SID of the recording to which the AddOnResult resource belongs. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related resources identified by their relative URIs. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


