# \Api20100401AccountApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_account**](Api20100401AccountApi.md#create_account) | **POST** /2010-04-01/Accounts.json | Create a new Twilio Subaccount from the account making the request
[**fetch_account**](Api20100401AccountApi.md#fetch_account) | **GET** /2010-04-01/Accounts/{Sid}.json | Fetch the account specified by the provided Account Sid
[**list_account**](Api20100401AccountApi.md#list_account) | **GET** /2010-04-01/Accounts.json | Retrieves a collection of Accounts belonging to the account used to make the request
[**update_account**](Api20100401AccountApi.md#update_account) | **POST** /2010-04-01/Accounts/{Sid}.json | Modify the properties of a given Account



## create_account

> models::ApiV2010Account create_account(friendly_name)
Create a new Twilio Subaccount from the account making the request

Create a new Twilio Subaccount from the account making the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**friendly_name** | Option<**String**> | A human readable description of the account to create, defaults to `SubAccount Created at {YYYY-MM-DD HH:MM meridian}` |  |

### Return type

[**models::ApiV2010Account**](api.v2010.account.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_account

> models::ApiV2010Account fetch_account(sid)
Fetch the account specified by the provided Account Sid

Fetch the account specified by the provided Account Sid

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sid** | **String** | The Account Sid that uniquely identifies the account to fetch | [required] |

### Return type

[**models::ApiV2010Account**](api.v2010.account.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_account

> models::ListAccountResponse list_account(friendly_name, status, page_size, page, page_token)
Retrieves a collection of Accounts belonging to the account used to make the request

Retrieves a collection of Accounts belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**friendly_name** | Option<**String**> | Only return the Account resources with friendly names that exactly match this name. |  |
**status** | Option<[**AccountEnumStatus**](AccountEnumStatus.md)> | Only return Account resources with the given status. Can be `closed`, `suspended` or `active`. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListAccountResponse**](ListAccountResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_account

> models::ApiV2010Account update_account(sid, friendly_name, status)
Modify the properties of a given Account

Modify the properties of a given Account

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**sid** | **String** | The Account Sid that uniquely identifies the account to update | [required] |
**friendly_name** | Option<**String**> | Update the human-readable description of this Account |  |
**status** | Option<[**models::AccountEnumStatus**](AccountEnumStatus.md)> |  |  |

### Return type

[**models::ApiV2010Account**](api.v2010.account.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

