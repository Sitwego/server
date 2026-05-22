# \Api20100401DataApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_recording_add_on_result_payload_data**](Api20100401DataApi.md#fetch_recording_add_on_result_payload_data) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{ReferenceSid}/AddOnResults/{AddOnResultSid}/Payloads/{PayloadSid}/Data.json | Fetch an instance of a result payload



## fetch_recording_add_on_result_payload_data

> fetch_recording_add_on_result_payload_data(account_sid, reference_sid, add_on_result_sid, payload_sid)
Fetch an instance of a result payload

Fetch an instance of a result payload

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Recording AddOnResult Payload resource to fetch. | [required] |
**reference_sid** | **String** | The SID of the recording to which the AddOnResult resource that contains the payload to fetch belongs. | [required] |
**add_on_result_sid** | **String** | The SID of the AddOnResult to which the payload to fetch belongs. | [required] |
**payload_sid** | **String** | The Twilio-provided string that uniquely identifies the Recording AddOnResult Payload resource to fetch. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

