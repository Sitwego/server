# \Api20100401PayloadApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_recording_add_on_result_payload**](Api20100401PayloadApi.md#delete_recording_add_on_result_payload) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults/{AddOnResultSid}/Payloads/{Sid}.json | Delete a payload from the result along with all associated Data
[**fetch_recording_add_on_result_payload**](Api20100401PayloadApi.md#fetch_recording_add_on_result_payload) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults/{AddOnResultSid}/Payloads/{Sid}.json | Fetch an instance of a result payload
[**list_recording_add_on_result_payload**](Api20100401PayloadApi.md#list_recording_add_on_result_payload) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults/{AddOnResultSid}/Payloads.json | Retrieve a list of payloads belonging to the AddOnResult



## delete_recording_add_on_result_payload

> delete_recording_add_on_result_payload(account_sid, reference_sid, add_on_result_sid, sid)
Delete a payload from the result along with all associated Data

Delete a payload from the result along with all associated Data

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult Payload resources to delete. | [required] |
**reference_sid** | **String** | The SID of the recording to which the AddOnResult resource that contains the payloads to delete belongs. | [required] |
**add_on_result_sid** | **String** | The SID of the AddOnResult to which the payloads to delete belongs. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Recording AddOnResult Payload resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_recording_add_on_result_payload

> models::ApiV2010AccountRecordingRecordingAddOnResultRecordingAddOnResultPayload fetch_recording_add_on_result_payload(account_sid, reference_sid, add_on_result_sid, sid)
Fetch an instance of a result payload

Fetch an instance of a result payload

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult Payload resource to fetch. | [required] |
**reference_sid** | **String** | The SID of the recording to which the AddOnResult resource that contains the payload to fetch belongs. | [required] |
**add_on_result_sid** | **String** | The SID of the AddOnResult to which the payload to fetch belongs. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Recording AddOnResult Payload resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountRecordingRecordingAddOnResultRecordingAddOnResultPayload**](api.v2010.account.recording.recording_add_on_result.recording_add_on_result_payload.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_recording_add_on_result_payload

> models::ListRecordingAddOnResultPayloadResponse list_recording_add_on_result_payload(account_sid, reference_sid, add_on_result_sid, page_size, page, page_token)
Retrieve a list of payloads belonging to the AddOnResult

Retrieve a list of payloads belonging to the AddOnResult

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult Payload resources to read. | [required] |
**reference_sid** | **String** | The SID of the recording to which the AddOnResult resource that contains the payloads to read belongs. | [required] |
**add_on_result_sid** | **String** | The SID of the AddOnResult to which the payloads to read belongs. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListRecordingAddOnResultPayloadResponse**](ListRecordingAddOnResultPayloadResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

