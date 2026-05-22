# ApiV2010AccountRecordingRecordingTranscription

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resource. | [optional]
**api_version** | Option<**String**> | The API version used to create the transcription. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**duration** | Option<**String**> | The duration of the transcribed audio in seconds. | [optional]
**price** | Option<**f64**> | The charge for the transcript in the currency associated with the account. This value is populated after the transcript is complete so it may not be available immediately. | [optional]
**price_unit** | Option<**String**> | The currency in which `price` is measured, in [ISO 4127](https://www.iso.org/iso/home/standards/currency_codes.htm) format (e.g. `usd`, `eur`, `jpy`). | [optional]
**recording_sid** | Option<**String**> | The SID of the [Recording](https://www.twilio.com/docs/voice/api/recording) from which the transcription was created. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the Transcription resource. | [optional]
**status** | Option<[**models::RecordingTranscriptionEnumStatus**](RecordingTranscriptionEnumStatus.md)> |  | [optional]
**transcription_text** | Option<**String**> | The text content of the transcription. | [optional]
**r#type** | Option<**String**> | The transcription type. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


