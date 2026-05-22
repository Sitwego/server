# \Api20100401MemberApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_member**](Api20100401MemberApi.md#fetch_member) | **GET** /2010-04-01/Accounts/{AccountSid}/Queues/{QueueSid}/Members/{CallSid}.json | Fetch a specific member from the queue
[**list_member**](Api20100401MemberApi.md#list_member) | **GET** /2010-04-01/Accounts/{AccountSid}/Queues/{QueueSid}/Members.json | Retrieve the members of the queue
[**update_member**](Api20100401MemberApi.md#update_member) | **POST** /2010-04-01/Accounts/{AccountSid}/Queues/{QueueSid}/Members/{CallSid}.json | Dequeue a member from a queue and have the member's call begin executing the TwiML document at that URL



## fetch_member

> models::ApiV2010AccountQueueMember fetch_member(account_sid, queue_sid, call_sid)
Fetch a specific member from the queue

Fetch a specific member from the queue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Member resource(s) to fetch. | [required] |
**queue_sid** | **String** | The SID of the Queue in which to find the members to fetch. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID of the resource(s) to fetch. | [required] |

### Return type

[**models::ApiV2010AccountQueueMember**](api.v2010.account.queue.member.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_member

> models::ListMemberResponse list_member(account_sid, queue_sid, page_size, page, page_token)
Retrieve the members of the queue

Retrieve the members of the queue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Member resource(s) to read. | [required] |
**queue_sid** | **String** | The SID of the Queue in which to find the members | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListMemberResponse**](ListMemberResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_member

> models::ApiV2010AccountQueueMember update_member(account_sid, queue_sid, call_sid, url, method)
Dequeue a member from a queue and have the member's call begin executing the TwiML document at that URL

Dequeue a member from a queue and have the member's call begin executing the TwiML document at that URL

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Member resource(s) to update. | [required] |
**queue_sid** | **String** | The SID of the Queue in which to find the members to update. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID of the resource(s) to update. | [required] |
**url** | **String** | The absolute URL of the Queue resource. | [required] |
**method** | Option<**String**> | How to pass the update request data. Can be `GET` or `POST` and the default is `POST`. `POST` sends the data as encoded form data and `GET` sends the data as query parameters. |  |

### Return type

[**models::ApiV2010AccountQueueMember**](api.v2010.account.queue.member.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

