# \Api20100401SigningKeyApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_signing_key**](Api20100401SigningKeyApi.md#delete_signing_key) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SigningKeys/{Sid}.json | 
[**fetch_signing_key**](Api20100401SigningKeyApi.md#fetch_signing_key) | **GET** /2010-04-01/Accounts/{AccountSid}/SigningKeys/{Sid}.json | 
[**list_signing_key**](Api20100401SigningKeyApi.md#list_signing_key) | **GET** /2010-04-01/Accounts/{AccountSid}/SigningKeys.json | 
[**update_signing_key**](Api20100401SigningKeyApi.md#update_signing_key) | **POST** /2010-04-01/Accounts/{AccountSid}/SigningKeys/{Sid}.json | 



## delete_signing_key

> delete_signing_key(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** |  | [required] |
**sid** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_signing_key

> models::ApiV2010AccountSigningKey fetch_signing_key(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** |  | [required] |
**sid** | **String** |  | [required] |

### Return type

[**models::ApiV2010AccountSigningKey**](api.v2010.account.signing_key.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_signing_key

> models::ListSigningKeyResponse list_signing_key(account_sid, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** |  | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSigningKeyResponse**](ListSigningKeyResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_signing_key

> models::ApiV2010AccountSigningKey update_signing_key(account_sid, sid, friendly_name)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** |  | [required] |
**sid** | **String** |  | [required] |
**friendly_name** | Option<**String**> |  |  |

### Return type

[**models::ApiV2010AccountSigningKey**](api.v2010.account.signing_key.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

