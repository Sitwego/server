# \Api20100401QueueApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_queue**](Api20100401QueueApi.md#create_queue) | **POST** /2010-04-01/Accounts/{AccountSid}/Queues.json | Create a queue
[**delete_queue**](Api20100401QueueApi.md#delete_queue) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Queues/{Sid}.json | Remove an empty queue
[**fetch_queue**](Api20100401QueueApi.md#fetch_queue) | **GET** /2010-04-01/Accounts/{AccountSid}/Queues/{Sid}.json | Fetch an instance of a queue identified by the QueueSid
[**list_queue**](Api20100401QueueApi.md#list_queue) | **GET** /2010-04-01/Accounts/{AccountSid}/Queues.json | Retrieve a list of queues belonging to the account used to make the request
[**update_queue**](Api20100401QueueApi.md#update_queue) | **POST** /2010-04-01/Accounts/{AccountSid}/Queues/{Sid}.json | Update the queue with the new parameters



## create_queue

> models::ApiV2010AccountQueue create_queue(account_sid, friendly_name, max_size)
Create a queue

Create a queue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**friendly_name** | **String** | A descriptive string that you created to describe this resource. It can be up to 64 characters long. | [required] |
**max_size** | Option<**i32**> | The maximum number of calls allowed to be in the queue. The default is 1000. The maximum is 5000. |  |

### Return type

[**models::ApiV2010AccountQueue**](api.v2010.account.queue.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_queue

> delete_queue(account_sid, sid)
Remove an empty queue

Remove an empty queue

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Queue resource to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Queue resource to delete | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_queue

> models::ApiV2010AccountQueue fetch_queue(account_sid, sid)
Fetch an instance of a queue identified by the QueueSid

Fetch an instance of a queue identified by the QueueSid

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Queue resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Queue resource to fetch | [required] |

### Return type

[**models::ApiV2010AccountQueue**](api.v2010.account.queue.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_queue

> models::ListQueueResponse list_queue(account_sid, page_size, page, page_token)
Retrieve a list of queues belonging to the account used to make the request

Retrieve a list of queues belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Queue resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListQueueResponse**](ListQueueResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_queue

> models::ApiV2010AccountQueue update_queue(account_sid, sid, friendly_name, max_size)
Update the queue with the new parameters

Update the queue with the new parameters

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Queue resource to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Queue resource to update | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe this resource. It can be up to 64 characters long. |  |
**max_size** | Option<**i32**> | The maximum number of calls allowed to be in the queue. The default is 1000. The maximum is 5000. |  |

### Return type

[**models::ApiV2010AccountQueue**](api.v2010.account.queue.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

