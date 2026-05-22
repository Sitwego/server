# \Api20100401RecordingApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_recording**](Api20100401RecordingApi.md#delete_recording) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Recordings/{Sid}.json | Delete a recording from your account
[**fetch_recording**](Api20100401RecordingApi.md#fetch_recording) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{Sid}.json | Fetch an instance of a recording
[**list_recording**](Api20100401RecordingApi.md#list_recording) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings.json | Retrieve a list of recordings belonging to the account used to make the request



## delete_recording

> delete_recording(account_sid, sid)
Delete a recording from your account

Delete a recording from your account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Recording resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_recording

> models::ApiV2010AccountRecording fetch_recording(account_sid, sid, include_soft_deleted)
Fetch an instance of a recording

Fetch an instance of a recording

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Recording resource to fetch. | [required] |
**include_soft_deleted** | Option<**bool**> | A boolean parameter indicating whether to retrieve soft deleted recordings or not. Recordings metadata are kept after deletion for a retention period of 40 days. |  |

### Return type

[**models::ApiV2010AccountRecording**](api.v2010.account.recording.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_recording

> models::ListRecordingResponse list_recording(account_sid, date_created, date_created_less_than, date_created_greater_than, call_sid, conference_sid, include_soft_deleted, page_size, page, page_token)
Retrieve a list of recordings belonging to the account used to make the request

Retrieve a list of recordings belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording resources to read. | [required] |
**date_created** | Option<**String**> | Only include recordings that were created on this date. Specify a date as `YYYY-MM-DD` in GMT, for example: `2009-07-06`, to read recordings that were created on this date. You can also specify an inequality, such as `DateCreated<=YYYY-MM-DD`, to read recordings that were created on or before midnight of this date, and `DateCreated>=YYYY-MM-DD` to read recordings that were created on or after midnight of this date. |  |
**date_created_less_than** | Option<**String**> | Only include recordings that were created on this date. Specify a date as `YYYY-MM-DD` in GMT, for example: `2009-07-06`, to read recordings that were created on this date. You can also specify an inequality, such as `DateCreated<=YYYY-MM-DD`, to read recordings that were created on or before midnight of this date, and `DateCreated>=YYYY-MM-DD` to read recordings that were created on or after midnight of this date. |  |
**date_created_greater_than** | Option<**String**> | Only include recordings that were created on this date. Specify a date as `YYYY-MM-DD` in GMT, for example: `2009-07-06`, to read recordings that were created on this date. You can also specify an inequality, such as `DateCreated<=YYYY-MM-DD`, to read recordings that were created on or before midnight of this date, and `DateCreated>=YYYY-MM-DD` to read recordings that were created on or after midnight of this date. |  |
**call_sid** | Option<**String**> | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID of the resources to read. |  |
**conference_sid** | Option<**String**> | The Conference SID that identifies the conference associated with the recording to read. |  |
**include_soft_deleted** | Option<**bool**> | A boolean parameter indicating whether to retrieve soft deleted recordings or not. Recordings metadata are kept after deletion for a retention period of 40 days. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListRecordingResponse**](ListRecordingResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

