# \Api20100401UserDefinedMessageApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_user_defined_message**](Api20100401UserDefinedMessageApi.md#create_user_defined_message) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/UserDefinedMessages.json | Create a new User Defined Message for the given Call SID.



## create_user_defined_message

> models::ApiV2010AccountCallUserDefinedMessage create_user_defined_message(account_sid, call_sid, content, idempotency_key)
Create a new User Defined Message for the given Call SID.

Create a new User Defined Message for the given Call SID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created User Defined Message. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the User Defined Message is associated with. | [required] |
**content** | **String** | The User Defined Message in the form of URL-encoded JSON string. | [required] |
**idempotency_key** | Option<**String**> | A unique string value to identify API call. This should be a unique string value per API call and can be a randomly generated. |  |

### Return type

[**models::ApiV2010AccountCallUserDefinedMessage**](api.v2010.account.call.user_defined_message.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

