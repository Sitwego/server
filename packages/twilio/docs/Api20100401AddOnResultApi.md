# \Api20100401AddOnResultApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_recording_add_on_result**](Api20100401AddOnResultApi.md#delete_recording_add_on_result) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults/{Sid}.json | Delete a result and purge all associated Payloads
[**fetch_recording_add_on_result**](Api20100401AddOnResultApi.md#fetch_recording_add_on_result) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults/{Sid}.json | Fetch an instance of an AddOnResult
[**list_recording_add_on_result**](Api20100401AddOnResultApi.md#list_recording_add_on_result) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults.json | Retrieve a list of results belonging to the recording



## delete_recording_add_on_result

> delete_recording_add_on_result(account_sid, reference_sid, sid)
Delete a result and purge all associated Payloads

Delete a result and purge all associated Payloads

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult resources to delete. | [required] |
**reference_sid** | **String** | The SID of the recording to which the result to delete belongs. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Recording AddOnResult resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_recording_add_on_result

> models::ApiV2010AccountRecordingRecordingAddOnResult fetch_recording_add_on_result(account_sid, reference_sid, sid)
Fetch an instance of an AddOnResult

Fetch an instance of an AddOnResult

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult resource to fetch. | [required] |
**reference_sid** | **String** | The SID of the recording to which the result to fetch belongs. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Recording AddOnResult resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountRecordingRecordingAddOnResult**](api.v2010.account.recording.recording_add_on_result.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_recording_add_on_result

> models::ListRecordingAddOnResultResponse list_recording_add_on_result(account_sid, reference_sid, page_size, page, page_token)
Retrieve a list of results belonging to the recording

Retrieve a list of results belonging to the recording

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult resources to read. | [required] |
**reference_sid** | **String** | The SID of the recording to which the result to read belongs. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListRecordingAddOnResultResponse**](ListRecordingAddOnResultResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

