# \Api20100401EventApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_call_event**](Api20100401EventApi.md#list_call_event) | **GET** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Events.json | Retrieve a list of all events for a call.



## list_call_event

> models::ListCallEventResponse list_call_event(account_sid, call_sid, page_size, page, page_token)
Retrieve a list of all events for a call.

Retrieve a list of all events for a call.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique SID identifier of the Account. | [required] |
**call_sid** | **String** | The unique SID identifier of the Call. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListCallEventResponse**](ListCallEventResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

