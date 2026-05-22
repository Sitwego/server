# \Api20100401CallNotificationApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_call_notification**](Api20100401CallNotificationApi.md#fetch_call_notification) | **GET** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Notifications/{Sid}.json | 
[**list_call_notification**](Api20100401CallNotificationApi.md#list_call_notification) | **GET** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Notifications.json | 



## fetch_call_notification

> models::ApiV2010AccountCallCallNotificationInstance fetch_call_notification(account_sid, call_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Call Notification resource to fetch. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID of the Call Notification resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Call Notification resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountCallCallNotificationInstance**](api.v2010.account.call.call_notification-instance.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_call_notification

> models::ListCallNotificationResponse list_call_notification(account_sid, call_sid, log, message_date, message_date_less_than, message_date_greater_than, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Call Notification resources to read. | [required] |
**call_sid** | **String** | The [Call](https://www.twilio.com/docs/voice/api/call-resource) SID of the Call Notification resources to read. | [required] |
**log** | Option<**i32**> | Only read notifications of the specified log level. Can be:  `0` to read only ERROR notifications or `1` to read only WARNING notifications. By default, all notifications are read. |  |
**message_date** | Option<**String**> | Only show notifications for the specified date, formatted as `YYYY-MM-DD`. You can also specify an inequality, such as `<=YYYY-MM-DD` for messages logged at or before midnight on a date, or `>=YYYY-MM-DD` for messages logged at or after midnight on a date. |  |
**message_date_less_than** | Option<**String**> | Only show notifications for the specified date, formatted as `YYYY-MM-DD`. You can also specify an inequality, such as `<=YYYY-MM-DD` for messages logged at or before midnight on a date, or `>=YYYY-MM-DD` for messages logged at or after midnight on a date. |  |
**message_date_greater_than** | Option<**String**> | Only show notifications for the specified date, formatted as `YYYY-MM-DD`. You can also specify an inequality, such as `<=YYYY-MM-DD` for messages logged at or before midnight on a date, or `>=YYYY-MM-DD` for messages logged at or after midnight on a date. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListCallNotificationResponse**](ListCallNotificationResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

