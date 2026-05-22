# \Api20100401AddressApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_address**](Api20100401AddressApi.md#create_address) | **POST** /2010-04-01/Accounts/{AccountSid}/Addresses.json | 
[**delete_address**](Api20100401AddressApi.md#delete_address) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Addresses/{Sid}.json | 
[**fetch_address**](Api20100401AddressApi.md#fetch_address) | **GET** /2010-04-01/Accounts/{AccountSid}/Addresses/{Sid}.json | 
[**list_address**](Api20100401AddressApi.md#list_address) | **GET** /2010-04-01/Accounts/{AccountSid}/Addresses.json | 
[**update_address**](Api20100401AddressApi.md#update_address) | **POST** /2010-04-01/Accounts/{AccountSid}/Addresses/{Sid}.json | 



## create_address

> models::ApiV2010AccountAddress create_address(account_sid, customer_name, street, city, region, postal_code, iso_country, friendly_name, emergency_enabled, auto_correct_address, street_secondary)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will be responsible for the new Address resource. | [required] |
**customer_name** | **String** | The name to associate with the new address. | [required] |
**street** | **String** | The number and street address of the new address. | [required] |
**city** | **String** | The city of the new address. | [required] |
**region** | **String** | The state or region of the new address. | [required] |
**postal_code** | **String** | The postal code of the new address. | [required] |
**iso_country** | **String** | The ISO country code of the new address. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the new address. It can be up to 64 characters long for Regulatory Compliance addresses and 32 characters long for Emergency addresses. |  |
**emergency_enabled** | Option<**bool**> | Whether to enable emergency calling on the new address. Can be: `true` or `false`. |  |
**auto_correct_address** | Option<**bool**> | Whether we should automatically correct the address. Can be: `true` or `false` and the default is `true`. If empty or `true`, we will correct the address you provide if necessary. If `false`, we won't alter the address you provide. |  |
**street_secondary** | Option<**String**> | The additional number and street address of the address. |  |

### Return type

[**models::ApiV2010AccountAddress**](api.v2010.account.address.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_address

> delete_address(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is responsible for the Address resource to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Address resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_address

> models::ApiV2010AccountAddress fetch_address(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is responsible for the Address resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Address resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountAddress**](api.v2010.account.address.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_address

> models::ListAddressResponse list_address(account_sid, customer_name, friendly_name, emergency_enabled, iso_country, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is responsible for the Address resource to read. | [required] |
**customer_name** | Option<**String**> | The `customer_name` of the Address resources to read. |  |
**friendly_name** | Option<**String**> | The string that identifies the Address resources to read. |  |
**emergency_enabled** | Option<**bool**> | Whether the address can be associated to a number for emergency calling. |  |
**iso_country** | Option<**String**> | The ISO country code of the Address resources to read. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListAddressResponse**](ListAddressResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_address

> models::ApiV2010AccountAddress update_address(account_sid, sid, friendly_name, customer_name, street, city, region, postal_code, emergency_enabled, auto_correct_address, street_secondary)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is responsible for the Address resource to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Address resource to update. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the new address. It can be up to 64 characters long for Regulatory Compliance addresses and 32 characters long for Emergency addresses. |  |
**customer_name** | Option<**String**> | The name to associate with the address. |  |
**street** | Option<**String**> | The number and street address of the address. |  |
**city** | Option<**String**> | The city of the address. |  |
**region** | Option<**String**> | The state or region of the address. |  |
**postal_code** | Option<**String**> | The postal code of the address. |  |
**emergency_enabled** | Option<**bool**> | Whether to enable emergency calling on the address. Can be: `true` or `false`. |  |
**auto_correct_address** | Option<**bool**> | Whether we should automatically correct the address. Can be: `true` or `false` and the default is `true`. If empty or `true`, we will correct the address you provide if necessary. If `false`, we won't alter the address you provide. |  |
**street_secondary** | Option<**String**> | The additional number and street address of the address. |  |

### Return type

[**models::ApiV2010AccountAddress**](api.v2010.account.address.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

