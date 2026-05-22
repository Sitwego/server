# \Api20100401TokenApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_token**](Api20100401TokenApi.md#create_token) | **POST** /2010-04-01/Accounts/{AccountSid}/Tokens.json | Create a new token for ICE servers



## create_token

> models::ApiV2010AccountToken create_token(account_sid, ttl)
Create a new token for ICE servers

Create a new token for ICE servers

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**ttl** | Option<**i32**> | The duration in seconds for which the generated credentials are valid. The default value is 86400 (24 hours). |  |

### Return type

[**models::ApiV2010AccountToken**](api.v2010.account.token.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

