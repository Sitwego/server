# \Api20100401ConferenceRecordingApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_conference_recording**](Api20100401ConferenceRecordingApi.md#delete_conference_recording) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Recordings/{Sid}.json | Delete a recording from your account
[**fetch_conference_recording**](Api20100401ConferenceRecordingApi.md#fetch_conference_recording) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Recordings/{Sid}.json | Fetch an instance of a recording for a call
[**list_conference_recording**](Api20100401ConferenceRecordingApi.md#list_conference_recording) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Recordings.json | Retrieve a list of recordings belonging to the call used to make the request
[**update_conference_recording**](Api20100401ConferenceRecordingApi.md#update_conference_recording) | **POST** /2010-04-01/Accounts/{AccountSid}/Conferences/{ConferenceSid}/Recordings/{Sid}.json | Changes the status of the recording to paused, stopped, or in-progress. Note: To use `Twilio.CURRENT`, pass it as recording sid.



## delete_conference_recording

> delete_conference_recording(account_sid, conference_sid, sid)
Delete a recording from your account

Delete a recording from your account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference Recording resources to delete. | [required] |
**conference_sid** | **String** | The Conference SID that identifies the conference associated with the recording to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference Recording resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_conference_recording

> models::ApiV2010AccountConferenceConferenceRecording fetch_conference_recording(account_sid, conference_sid, sid)
Fetch an instance of a recording for a call

Fetch an instance of a recording for a call

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference Recording resource to fetch. | [required] |
**conference_sid** | **String** | The Conference SID that identifies the conference associated with the recording to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference Recording resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountConferenceConferenceRecording**](api.v2010.account.conference.conference_recording.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_conference_recording

> models::ListConferenceRecordingResponse list_conference_recording(account_sid, conference_sid, date_created, date_created_less_than, date_created_greater_than, page_size, page, page_token)
Retrieve a list of recordings belonging to the call used to make the request

Retrieve a list of recordings belonging to the call used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference Recording resources to read. | [required] |
**conference_sid** | **String** | The Conference SID that identifies the conference associated with the recording to read. | [required] |
**date_created** | Option<**String**> | The `date_created` value, specified as `YYYY-MM-DD`, of the resources to read. You can also specify inequality: `DateCreated<=YYYY-MM-DD` will return recordings generated at or before midnight on a given date, and `DateCreated>=YYYY-MM-DD` returns recordings generated at or after midnight on a date. |  |
**date_created_less_than** | Option<**String**> | The `date_created` value, specified as `YYYY-MM-DD`, of the resources to read. You can also specify inequality: `DateCreated<=YYYY-MM-DD` will return recordings generated at or before midnight on a given date, and `DateCreated>=YYYY-MM-DD` returns recordings generated at or after midnight on a date. |  |
**date_created_greater_than** | Option<**String**> | The `date_created` value, specified as `YYYY-MM-DD`, of the resources to read. You can also specify inequality: `DateCreated<=YYYY-MM-DD` will return recordings generated at or before midnight on a given date, and `DateCreated>=YYYY-MM-DD` returns recordings generated at or after midnight on a date. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListConferenceRecordingResponse**](ListConferenceRecordingResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_conference_recording

> models::ApiV2010AccountConferenceConferenceRecording update_conference_recording(account_sid, conference_sid, sid, status, pause_behavior)
Changes the status of the recording to paused, stopped, or in-progress. Note: To use `Twilio.CURRENT`, pass it as recording sid.

Changes the status of the recording to paused, stopped, or in-progress. Note: To use `Twilio.CURRENT`, pass it as recording sid.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference Recording resource to update. | [required] |
**conference_sid** | **String** | The Conference SID that identifies the conference associated with the recording to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference Recording resource to update. Use `Twilio.CURRENT` to reference the current active recording. | [required] |
**status** | [**models::ConferenceRecordingEnumStatus**](ConferenceRecordingEnumStatus.md) |  | [required] |
**pause_behavior** | Option<**String**> | Whether to record during a pause. Can be: `skip` or `silence` and the default is `silence`. `skip` does not record during the pause period, while `silence` will replace the actual audio of the call with silence during the pause period. This parameter only applies when setting `status` is set to `paused`. |  |

### Return type

[**models::ApiV2010AccountConferenceConferenceRecording**](api.v2010.account.conference.conference_recording.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

