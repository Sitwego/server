# \Api20100401AvailablePhoneNumberCountryApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_available_phone_number_country**](Api20100401AvailablePhoneNumberCountryApi.md#fetch_available_phone_number_country) | **GET** /2010-04-01/Accounts/{AccountSid}/AvailablePhoneNumbers/{CountryCode}.json | 
[**list_available_phone_number_country**](Api20100401AvailablePhoneNumberCountryApi.md#list_available_phone_number_country) | **GET** /2010-04-01/Accounts/{AccountSid}/AvailablePhoneNumbers.json | 



## fetch_available_phone_number_country

> models::ApiV2010AccountAvailablePhoneNumberCountry fetch_available_phone_number_country(account_sid, country_code)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) requesting the available phone number Country resource. | [required] |
**country_code** | **String** | The [ISO-3166-1](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) country code of the country to fetch available phone number information about. | [required] |

### Return type

[**models::ApiV2010AccountAvailablePhoneNumberCountry**](api.v2010.account.available_phone_number_country.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_available_phone_number_country

> models::ListAvailablePhoneNumberCountryResponse list_available_phone_number_country(account_sid, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) requesting the available phone number Country resources. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListAvailablePhoneNumberCountryResponse**](ListAvailablePhoneNumberCountryResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

