# \Api20100401NewKeyApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_new_key**](Api20100401NewKeyApi.md#create_new_key) | **POST** /2010-04-01/Accounts/{AccountSid}/Keys.json | 



## create_new_key

> models::ApiV2010AccountNewKey create_new_key(account_sid, friendly_name)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will be responsible for the new Key resource. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |

### Return type

[**models::ApiV2010AccountNewKey**](api.v2010.account.new_key.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

