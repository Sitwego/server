# \Api20100401IpAccessControlListMappingApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_sip_ip_access_control_list_mapping**](Api20100401IpAccessControlListMappingApi.md#create_sip_ip_access_control_list_mapping) | **POST** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/IpAccessControlListMappings.json | Create a new IpAccessControlListMapping resource.
[**delete_sip_ip_access_control_list_mapping**](Api20100401IpAccessControlListMappingApi.md#delete_sip_ip_access_control_list_mapping) | **DELETE** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/IpAccessControlListMappings/{Sid}.json | Delete an IpAccessControlListMapping resource.
[**fetch_sip_ip_access_control_list_mapping**](Api20100401IpAccessControlListMappingApi.md#fetch_sip_ip_access_control_list_mapping) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/IpAccessControlListMappings/{Sid}.json | Fetch an IpAccessControlListMapping resource.
[**list_sip_ip_access_control_list_mapping**](Api20100401IpAccessControlListMappingApi.md#list_sip_ip_access_control_list_mapping) | **GET** /2010-04-01/Accounts/{AccountSid}/SIP/Domains/{DomainSid}/IpAccessControlListMappings.json | Retrieve a list of IpAccessControlListMapping resources.



## create_sip_ip_access_control_list_mapping

> models::ApiV2010AccountSipSipDomainSipIpAccessControlListMapping create_sip_ip_access_control_list_mapping(account_sid, domain_sid, ip_access_control_list_sid)
Create a new IpAccessControlListMapping resource.

Create a new IpAccessControlListMapping resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP domain. | [required] |
**ip_access_control_list_sid** | **String** | The unique id of the IP access control list to map to the SIP domain. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomainSipIpAccessControlListMapping**](api.v2010.account.sip.sip_domain.sip_ip_access_control_list_mapping.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_sip_ip_access_control_list_mapping

> delete_sip_ip_access_control_list_mapping(account_sid, domain_sid, sid)
Delete an IpAccessControlListMapping resource.

Delete an IpAccessControlListMapping resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP domain. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_sip_ip_access_control_list_mapping

> models::ApiV2010AccountSipSipDomainSipIpAccessControlListMapping fetch_sip_ip_access_control_list_mapping(account_sid, domain_sid, sid)
Fetch an IpAccessControlListMapping resource.

Fetch an IpAccessControlListMapping resource.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP domain. | [required] |
**sid** | **String** | A 34 character string that uniquely identifies the resource to fetch. | [required] |

### Return type

[**models::ApiV2010AccountSipSipDomainSipIpAccessControlListMapping**](api.v2010.account.sip.sip_domain.sip_ip_access_control_list_mapping.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_sip_ip_access_control_list_mapping

> models::ListSipIpAccessControlListMappingResponse list_sip_ip_access_control_list_mapping(account_sid, domain_sid, page_size, page, page_token)
Retrieve a list of IpAccessControlListMapping resources.

Retrieve a list of IpAccessControlListMapping resources.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The unique id of the Account that is responsible for this resource. | [required] |
**domain_sid** | **String** | A 34 character string that uniquely identifies the SIP domain. | [required] |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListSipIpAccessControlListMappingResponse**](ListSipIpAccessControlListMappingResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

