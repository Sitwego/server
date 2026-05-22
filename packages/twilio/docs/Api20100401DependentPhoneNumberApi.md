# \Api20100401DependentPhoneNumberApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_dependent_phone_number**](Api20100401DependentPhoneNumberApi.md#list_dependent_phone_number) | **GET** /2010-04-01/Accounts/{AccountSid}/Addresses/{AddressSid}/DependentPhoneNumbers.json | 



## list_dependent_phone_number

> models::ListDependentPhoneNumberResponse list_dependent_phone_number(account_sid, address_sid, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the DependentPhoneNumber resources to read. | [required] |
**address_sid** | **String** | The SID of the Address resource associated with the phone number. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListDependentPhoneNumberResponse**](ListDependentPhoneNumberResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

