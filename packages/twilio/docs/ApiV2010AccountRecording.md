# ApiV2010AccountRecording

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording resource. | [optional]
**api_version** | Option<**String**> | The API version used during the recording. | [optional]
**call_sid** | Option<**String**> | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Recording resource is associated with. This will always refer to the parent leg of a two-leg call. | [optional]
**conference_sid** | Option<**String**> | The Conference SID that identifies the conference associated with the recording, if a conference recording. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**start_time** | Option<**String**> | The start time of the recording in GMT and in [RFC 2822](https://www.php.net/manual/en/class.datetime.php#datetime.constants.rfc2822) format. | [optional]
**duration** | Option<**String**> | The length of the recording in seconds. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the Recording resource. | [optional]
**price** | Option<**String**> | The one-time cost of creating the recording in the `price_unit` currency. | [optional]
**price_unit** | Option<**String**> | The currency used in the `price` property. Example: `USD`. | [optional]
**status** | Option<[**models::RecordingEnumStatus**](RecordingEnumStatus.md)> |  | [optional]
**channels** | Option<**i32**> | The number of channels in the recording resource. For information on specifying the number of channels in the downloaded recording file, check out [Fetch a Recording’s media file](https://www.twilio.com/docs/voice/api/recording#download-dual-channel-media-file). | [optional]
**source** | Option<[**models::RecordingEnumSource**](RecordingEnumSource.md)> |  | [optional]
**error_code** | Option<**i32**> | The error code that describes why the recording is `absent`. The error code is described in our [Error Dictionary](https://www.twilio.com/docs/api/errors). This value is null if the recording `status` is not `absent`. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**encryption_details** | Option<**serde_json::Value**> | How to decrypt the recording if it was encrypted using [Call Recording Encryption](https://www.twilio.com/docs/voice/tutorials/voice-recording-encryption) feature. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related resources identified by their relative URIs. | [optional]
**media_url** | Option<**String**> | The URL of the media file associated with this recording resource. When stored externally, this is the full URL location of the media file. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


