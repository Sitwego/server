# \Api20100401ValidationRequestApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_validation_request**](Api20100401ValidationRequestApi.md#create_validation_request) | **POST** /2010-04-01/Accounts/{AccountSid}/OutgoingCallerIds.json | 



## create_validation_request

> models::ApiV2010AccountValidationRequest create_validation_request(account_sid, phone_number, friendly_name, call_delay, extension, status_callback, status_callback_method)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) responsible for the new caller ID resource. | [required] |
**phone_number** | **String** | The phone number to verify in [E.164](https://www.twilio.com/docs/glossary/what-e164) format, which consists of a + followed by the country code and subscriber number. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the new caller ID resource. It can be up to 64 characters long. The default value is a formatted version of the phone number. |  |
**call_delay** | Option<**i32**> | The number of seconds to delay before initiating the verification call. Can be an integer between `0` and `60`, inclusive. The default is `0`. |  |
**extension** | Option<**String**> | The digits to dial after connecting the verification call. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information about the verification process to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` or `POST`, and the default is `POST`. |  |

### Return type

[**models::ApiV2010AccountValidationRequest**](api.v2010.account.validation_request.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

