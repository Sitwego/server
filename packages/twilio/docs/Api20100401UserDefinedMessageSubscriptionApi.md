# \Api20100401UserDefinedMessageSubscriptionApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_user_defined_message_subscription**](Api20100401UserDefinedMessageSubscriptionApi.md#create_user_defined_message_subscription) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/UserDefinedMessageSubscriptions.json | Subscribe to User Defined Messages for a given Call SID.
[**delete_user_defined_message_subscription**](Api20100401UserDefinedMessageSubscriptionApi.md#delete_user_defined_message_subscription) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/UserDefinedMessageSubscriptions/{Sid}.json | Delete a specific User Defined Message Subscription.



## create_user_defined_message_subscription

> models::ApiV2010AccountCallUserDefinedMessageSubscription create_user_defined_message_subscription(account_sid, call_sid, callback, idempotency_key, method)
Subscribe to User Defined Messages for a given Call SID.

Subscribe to User Defined Messages for a given Call SID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that subscribed to the User Defined Messages. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the User Defined Messages subscription is associated with. This refers to the Call SID that is producing the user defined messages. | [required] |
**callback** | **String** | The URL we should call using the `method` to send user defined events to your application. URLs must contain a valid hostname (underscores are not permitted). | [required] |
**idempotency_key** | Option<**String**> | A unique string value to identify API call. This should be a unique string value per API call and can be a randomly generated. |  |
**method** | Option<**String**> | The HTTP method Twilio will use when requesting the above `Url`. Either `GET` or `POST`. Default is `POST`. |  |

### Return type

[**models::ApiV2010AccountCallUserDefinedMessageSubscription**](api.v2010.account.call.user_defined_message_subscription.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user_defined_message_subscription

> delete_user_defined_message_subscription(account_sid, call_sid, sid)
Delete a specific User Defined Message Subscription.

Delete a specific User Defined Message Subscription.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that subscribed to the User Defined Messages. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the User Defined Message Subscription is associated with. This refers to the Call SID that is producing the User Defined Messages. | [required] |
**sid** | **String** | The SID that uniquely identifies this User Defined Message Subscription. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

