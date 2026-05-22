# \Api20100401KeyApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_key**](Api20100401KeyApi.md#delete_key) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Keys/{Sid}.json | 
[**fetch_key**](Api20100401KeyApi.md#fetch_key) | **GET** /2010-04-01/Accounts/{AccountSid}/Keys/{Sid}.json | 
[**list_key**](Api20100401KeyApi.md#list_key) | **GET** /2010-04-01/Accounts/{AccountSid}/Keys.json | 
[**update_key**](Api20100401KeyApi.md#update_key) | **POST** /2010-04-01/Accounts/{AccountSid}/Keys/{Sid}.json | 



## delete_key

> delete_key(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Key resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Key resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_key

> models::ApiV2010AccountKey fetch_key(account_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Key resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Key resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountKey**](api.v2010.account.key.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_key

> models::ListKeyResponse list_key(account_sid, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Key resources to read. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListKeyResponse**](ListKeyResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_key

> models::ApiV2010AccountKey update_key(account_sid, sid, friendly_name)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Key resources to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Key resource to update. | [required] |
**friendly_name** | Option<**String**> | A descriptive string that you create to describe the resource. It can be up to 64 characters long. |  |

### Return type

[**models::ApiV2010AccountKey**](api.v2010.account.key.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

