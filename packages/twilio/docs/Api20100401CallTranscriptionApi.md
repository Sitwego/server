# \Api20100401CallTranscriptionApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_realtime_transcription**](Api20100401CallTranscriptionApi.md#create_realtime_transcription) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Transcriptions.json | Create a Transcription
[**update_realtime_transcription**](Api20100401CallTranscriptionApi.md#update_realtime_transcription) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Transcriptions/{Sid}.json | Stop a Transcription using either the SID of the Transcription resource or the `name` used when creating the resource



## create_realtime_transcription

> models::ApiV2010AccountCallRealtimeTranscription create_realtime_transcription(account_sid, call_sid, name, track, status_callback_url, status_callback_method, inbound_track_label, outbound_track_label, partial_results, language_code, transcription_engine, profanity_filter, speech_model, hints, enable_automatic_punctuation, intelligence_service, conversation_configuration, conversation_id, enable_provider_data)
Create a Transcription

Create a Transcription

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Transcription resource. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Transcription resource is associated with. | [required] |
**name** | Option<**String**> | The user-specified name of this Transcription, if one was given when the Transcription was created. This may be used to stop the Transcription. |  |
**track** | Option<[**models::RealtimeTranscriptionEnumTrack**](RealtimeTranscriptionEnumTrack.md)> |  |  |
**status_callback_url** | Option<**String**> | Absolute URL of the status callback. |  |
**status_callback_method** | Option<**String**> | The http method for the status_callback (one of GET, POST). |  |
**inbound_track_label** | Option<**String**> | Friendly name given to the Inbound Track |  |
**outbound_track_label** | Option<**String**> | Friendly name given to the Outbound Track |  |
**partial_results** | Option<**bool**> | Indicates if partial results are going to be sent to the customer |  |
**language_code** | Option<**String**> | Language code used by the transcription engine, specified in [BCP-47](https://www.rfc-editor.org/rfc/bcp/bcp47.txt) format |  |
**transcription_engine** | Option<**String**> | Definition of the transcription engine to be used, among those supported by Twilio |  |
**profanity_filter** | Option<**bool**> | indicates if the server will attempt to filter out profanities, replacing all but the initial character in each filtered word with asterisks |  |
**speech_model** | Option<**String**> | Recognition model used by the transcription engine, among those supported by the provider |  |
**hints** | Option<**String**> | A Phrase contains words and phrase \\\"hints\\\" so that the speech recognition engine is more likely to recognize them. |  |
**enable_automatic_punctuation** | Option<**bool**> | The provider will add punctuation to recognition result |  |
**intelligence_service** | Option<**String**> | The SID or unique name of the [Intelligence Service](https://www.twilio.com/docs/conversational-intelligence/api/service-resource) for persisting transcripts and running post-call Language Operators |  |
**conversation_configuration** | Option<**String**> | The ID of the Conversations Configuration for customizing conversation behavior in Intelligence Service |  |
**conversation_id** | Option<**String**> | The ID of the Conversation for associating this Transcription with an existing Conversation in Intelligence Service |  |
**enable_provider_data** | Option<**bool**> | Whether the callback includes raw provider data. |  |

### Return type

[**models::ApiV2010AccountCallRealtimeTranscription**](api.v2010.account.call.realtime_transcription.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_realtime_transcription

> models::ApiV2010AccountCallRealtimeTranscription update_realtime_transcription(account_sid, call_sid, sid, status)
Stop a Transcription using either the SID of the Transcription resource or the `name` used when creating the resource

Stop a Transcription using either the SID of the Transcription resource or the `name` used when creating the resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Transcription resource. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Transcription resource is associated with. | [required] |
**sid** | **String** | The SID of the Transcription resource, or the `name` used when creating the resource | [required] |
**status** | [**models::RealtimeTranscriptionEnumUpdateStatus**](RealtimeTranscriptionEnumUpdateStatus.md) |  | [required] |

### Return type

[**models::ApiV2010AccountCallRealtimeTranscription**](api.v2010.account.call.realtime_transcription.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

