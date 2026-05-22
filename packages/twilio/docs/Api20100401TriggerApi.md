# \Api20100401TriggerApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_usage_trigger**](Api20100401TriggerApi.md#create_usage_trigger) | **POST** /2010-04-01/Accounts/{AccountSid}/Usage/Triggers.json | Create a new UsageTrigger
[**delete_usage_trigger**](Api20100401TriggerApi.md#delete_usage_trigger) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Usage/Triggers/{Sid}.json | 
[**fetch_usage_trigger**](Api20100401TriggerApi.md#fetch_usage_trigger) | **GET** /2010-04-01/Accounts/{AccountSid}/Usage/Triggers/{Sid}.json | Fetch and instance of a usage-trigger
[**list_usage_trigger**](Api20100401TriggerApi.md#list_usage_trigger) | **GET** /2010-04-01/Accounts/{AccountSid}/Usage/Triggers.json | Retrieve a list of usage-triggers belonging to the account used to make the request
[**update_usage_trigger**](Api20100401TriggerApi.md#update_usage_trigger) | **POST** /2010-04-01/Accounts/{AccountSid}/Usage/Triggers/{Sid}.json | Update an instance of a usage trigger



## create_usage_trigger

> models::ApiV2010AccountUsageUsageTrigger create_usage_trigger(account_sid, callback_url, trigger_value, usage_category, callback_method, friendly_name, recurring, trigger_by)
Create a new UsageTrigger

Create a new UsageTrigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**callback_url** | **String** | The URL we should call using `callback_method` when the trigger fires. | [required] |
**trigger_value** | **String** | The usage value at which the trigger should fire.  For convenience, you can use an offset value such as `+30` to specify a trigger_value that is 30 units more than the current usage value. Be sure to urlencode a `+` as `%2B`. | [required] |
**usage_category** | **String** | The usage category that the trigger should watch.  Use one of the supported [usage categories](https://www.twilio.com/docs/usage/api/usage-record#usage-categories) for this value. | [required] |
**callback_method** | Option<**String**> | The HTTP method we should use to call `callback_url`. Can be: `GET` or `POST` and the default is `POST`. |  |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |
**recurring** | Option<[**models::UsageTriggerEnumRecurring**](UsageTriggerEnumRecurring.md)> |  |  |
**trigger_by** | Option<[**models::UsageTriggerEnumTriggerField**](UsageTriggerEnumTriggerField.md)> |  |  |

### Return type

[**models::ApiV2010AccountUsageUsageTrigger**](api.v2010.account.usage.usage_trigger.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_usage_trigger

> delete_usage_trigger(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the UsageTrigger resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the UsageTrigger resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_usage_trigger

> models::ApiV2010AccountUsageUsageTrigger fetch_usage_trigger(account_sid, sid)
Fetch and instance of a usage-trigger

Fetch and instance of a usage-trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the UsageTrigger resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the UsageTrigger resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountUsageUsageTrigger**](api.v2010.account.usage.usage_trigger.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_usage_trigger

> models::ListUsageTriggerResponse list_usage_trigger(account_sid, recurring, trigger_by, usage_category, page_size, page, page_token)
Retrieve a list of usage-triggers belonging to the account used to make the request

Retrieve a list of usage-triggers belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the UsageTrigger resources to read. | [required] |
**recurring** | Option<[**UsageTriggerEnumRecurring**](UsageTriggerEnumRecurring.md)> | The frequency of recurring UsageTriggers to read. Can be: `daily`, `monthly`, or `yearly` to read recurring UsageTriggers. An empty value or a value of `alltime` reads non-recurring UsageTriggers. |  |
**trigger_by** | Option<[**UsageTriggerEnumTriggerField**](UsageTriggerEnumTriggerField.md)> | The trigger field of the UsageTriggers to read.  Can be: `count`, `usage`, or `price` as described in the [UsageRecords documentation](https://www.twilio.com/docs/usage/api/usage-record#usage-count-price). |  |
**usage_category** | Option<**String**> | The usage category of the UsageTriggers to read. Must be a supported [usage categories](https://www.twilio.com/docs/usage/api/usage-record#usage-categories). |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListUsageTriggerResponse**](ListUsageTriggerResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_usage_trigger

> models::ApiV2010AccountUsageUsageTrigger update_usage_trigger(account_sid, sid, callback_method, callback_url, friendly_name)
Update an instance of a usage trigger

Update an instance of a usage trigger

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the UsageTrigger resources to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the UsageTrigger resource to update. | [required] |
**callback_method** | Option<**String**> | The HTTP method we should use to call `callback_url`. Can be: `GET` or `POST` and the default is `POST`. |  |
**callback_url** | Option<**String**> | The URL we should call using `callback_method` when the trigger fires. |  |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |

### Return type

[**models::ApiV2010AccountUsageUsageTrigger**](api.v2010.account.usage.usage_trigger.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

